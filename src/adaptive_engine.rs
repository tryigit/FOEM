use crate::exec;
use crate::features::repair::{open_diag_port, send_at_command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FuzzGoal {
    EnableDiagPort,
    BypassRsaAuth,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepKind {
    AdbShell,
    Fastboot,
    AtCommand,
    RawDiag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitStep {
    pub kind: StepKind,
    pub payload: String,
    pub success_markers: Vec<String>,
    pub failure_markers: Vec<String>,
    #[serde(default)]
    pub retries: u8,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitRecipe {
    pub goal: FuzzGoal,
    pub name: String,
    pub steps: Vec<ExploitStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub learned: HashMap<String, ExploitStep>,
}

impl KnowledgeBase {
    pub fn load() -> Self {
        let path = kb_path();
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(kb) = serde_json::from_str::<KnowledgeBase>(&text) {
                return kb;
            }
        }
        KnowledgeBase {
            learned: HashMap::new(),
        }
    }

    pub fn save(&self) {
        let path = kb_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, serde_json::to_string_pretty(self).unwrap_or_default());
    }

    pub fn recall(&self, fingerprint: &str) -> Option<ExploitStep> {
        self.learned.get(fingerprint).cloned()
    }

    pub fn learn(&mut self, fingerprint: &str, step: ExploitStep) {
        self.learned.insert(fingerprint.to_string(), step);
        self.save();
    }
}


#[cfg(test)]
thread_local! {
    pub static MOCK_KB_PATH: std::cell::RefCell<Option<PathBuf>> = std::cell::RefCell::new(None);
}

fn kb_path() -> PathBuf {
    #[cfg(test)]
    {
        if let Some(path) = MOCK_KB_PATH.with(|m| m.borrow().clone()) {
            return path;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".foem")
            .join("learned_methods.json");
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile)
            .join(".foem")
            .join("learned_methods.json");
    }
    PathBuf::from(".foem/learned_methods.json")
}

pub fn fingerprint(model: &str, release: &str, platform: &str) -> String {
    format!("{}|{}|{}", model.trim(), release.trim(), platform.trim())
}

pub fn execute_goal(
    serial: &str,
    goal: FuzzGoal,
    fingerprint: &str,
    diag_port_hint: Option<&str>,
) -> String {
    let mut kb = KnowledgeBase::load();
    if let Some(step) = kb.recall(fingerprint) {
        if let Ok(out) = execute_step(serial, &step, diag_port_hint) {
            return format!("Used learned method for {}:\n{}", fingerprint, out);
        }
    }

    let recipes = builtin_recipes();
    let mut last_error = String::new();
    for recipe in recipes.iter().filter(|r| r.goal == goal) {
        for step in &recipe.steps {
            match execute_step(serial, step, diag_port_hint) {
                Ok(out) => {
                    if step.failure_markers.iter().any(|m| out.contains(m)) {
                        last_error =
                            format!("Step {} reported failure markers:\n{}", recipe.name, out);
                        continue;
                    }

                    if step.success_markers.iter().any(|m| out.contains(m))
                        || step.success_markers.is_empty()
                    {
                        kb.learn(fingerprint, step.clone());
                        return format!(
                            "{} succeeded via {}:\n{}",
                            goal_string(&goal),
                            recipe.name,
                            out
                        );
                    } else {
                        last_error = format!("No success markers matched:\n{}", out);
                    }
                }
                Err(e) => {
                    last_error = e;
                }
            }
        }
    }

    if last_error.is_empty() {
        "No matching recipe executed.".to_string()
    } else {
        format!("All recipes failed. Last error: {}", last_error)
    }
}

fn execute_step(
    serial: &str,
    step: &ExploitStep,
    diag_port_hint: Option<&str>,
) -> Result<String, String> {
    let retries = if step.retries == 0 {
        1
    } else {
        step.retries as usize
    };
    let mut last_error = String::new();
    for attempt in 0..retries {
        let res = std::panic::catch_unwind(|| match step.kind {
            StepKind::AdbShell => {
                let timeout = step
                    .timeout_ms
                    .map(Duration::from_millis)
                    .unwrap_or(exec::COMMAND_TIMEOUT);
                exec::run_with_timeout(
                    "adb",
                    &["-s", serial, "shell", &step.payload],
                    "ADB shell",
                    timeout,
                )
            }
            StepKind::Fastboot => {
                let timeout = step
                    .timeout_ms
                    .map(Duration::from_millis)
                    .unwrap_or(exec::COMMAND_TIMEOUT);
                exec::run_with_timeout(
                    "fastboot",
                    &["-s", serial, &step.payload],
                    "Fastboot",
                    timeout,
                )
            }
            StepKind::AtCommand => {
                let autodetected = autodetect_diag_port();
                let port_name = diag_port_hint
                    .or(autodetected.as_deref())
                    .ok_or_else(|| "No diagnostic port available for AT command".to_string())?;
                let mut port = open_diag_port(port_name)?;
                send_at_command(&mut port, &step.payload)
            }
            StepKind::RawDiag => {
                let autodetected = autodetect_diag_port();
                let port_name = diag_port_hint
                    .or(autodetected.as_deref())
                    .ok_or_else(|| "No diagnostic port available for DIAG command".to_string())?;
                let mut port = open_diag_port(port_name)?;
                let bytes = hex::decode(step.payload.replace([' ', '\n', '\r'], ""))
                    .map_err(|e| format!("Hex decode failed: {}", e))?;
                crate::features::repair::send_diag_bytes(&mut port, &bytes)
                    .map(|_| format!("Sent {} diag bytes", bytes.len()))
            }
        });

        match res {
            Ok(Ok(val)) => return Ok(val),
            Ok(Err(e)) => last_error = e,
            Err(_) => {
                last_error = format!("Step {:?} panicked during execution", step.kind);
            }
        };

        if attempt + 1 < retries {
            std::thread::sleep(Duration::from_millis(150));
        }
    }
    Err(last_error)
}

fn goal_string(goal: &FuzzGoal) -> String {
    match goal {
        FuzzGoal::EnableDiagPort => "EnableDiagPort".to_string(),
        FuzzGoal::BypassRsaAuth => "BypassRsaAuth".to_string(),
        FuzzGoal::Custom(s) => s.clone(),
    }
}

fn builtin_recipes() -> Vec<ExploitRecipe> {
    vec![
        ExploitRecipe {
            goal: FuzzGoal::EnableDiagPort,
            name: "USB Config Toggle".into(),
            steps: vec![ExploitStep {
                kind: StepKind::AdbShell,
                payload: "setprop sys.usb.config diag,adb".into(),
                success_markers: vec![],
                failure_markers: vec!["Permission denied".into()],
                retries: 0,
                timeout_ms: None,
            }],
        },
        ExploitRecipe {
            goal: FuzzGoal::EnableDiagPort,
            name: "Samsung Diag".into(),
            steps: vec![ExploitStep {
                kind: StepKind::AdbShell,
                payload: "setprop sys.usb.config diag,adb; setprop persist.sys.usb.config diag,adb"
                    .into(),
                success_markers: vec![],
                failure_markers: vec![],
                retries: 0,
                timeout_ms: None,
            }],
        },
        ExploitRecipe {
            goal: FuzzGoal::EnableDiagPort,
            name: "AT Diag Enable".into(),
            steps: vec![ExploitStep {
                kind: StepKind::AtCommand,
                payload: "AT+DIAG=1".into(),
                success_markers: vec!["OK".into()],
                failure_markers: vec!["ERROR".into()],
                retries: 0,
                timeout_ms: None,
            }],
        },
    ]
}

pub fn autodetect_diag_port() -> Option<String> {
    if let Ok(ports) = serialport::available_ports() {
        for p in ports {
            if matches!(p.port_type, serialport::SerialPortType::UsbPort(_)) {
                return Some(p.port_name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    struct KbPathMockGuard;

    impl Drop for KbPathMockGuard {
        fn drop(&mut self) {
            MOCK_KB_PATH.with(|m| *m.borrow_mut() = None);
        }
    }

    #[test]
    fn test_knowledge_base_save_and_load() -> Result<(), Box<dyn Error>> {
        let dir = std::env::temp_dir();
        let db_path = dir.join("test_kb_save.json");

        MOCK_KB_PATH.with(|m| *m.borrow_mut() = Some(db_path.clone()));
        let _guard = KbPathMockGuard; // Ensure cleanup

        let mut kb = KnowledgeBase {
            learned: HashMap::new(),
        };

        let step = ExploitStep {
            kind: StepKind::AdbShell,
            payload: "echo test".to_string(),
            success_markers: vec!["test".to_string()],
            failure_markers: vec![],
            retries: 1,
            timeout_ms: None,
        };
        kb.learn("test_fingerprint", step.clone());

        assert!(db_path.exists());

        let loaded_kb = KnowledgeBase::load();
        assert_eq!(loaded_kb.learned.len(), 1);
        let loaded_step = loaded_kb.recall("test_fingerprint").ok_or("missing step")?;
        assert_eq!(loaded_step.payload, "echo test");

        Ok(())
    }
}
