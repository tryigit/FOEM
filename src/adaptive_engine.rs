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
pub struct LearnedStep {
    pub recipe_name: String,
    pub step_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub learned: HashMap<String, LearnedStep>,
}

impl KnowledgeBase {
    pub fn load() -> Self {
        let path = kb_path();
        if let Ok(file) = fs::File::open(&path) {
            use std::io::Read;
            let mut reader = file.take(1024 * 1024); // 1 MB limit
            let mut text = String::new();
            if reader.read_to_string(&mut text).is_ok() {
                if let Ok(kb) = serde_json::from_str::<KnowledgeBase>(&text) {
                    return kb;
                }
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

    pub fn recall(&self, fingerprint: &str) -> Option<LearnedStep> {
        self.learned.get(fingerprint).cloned()
    }

    pub fn learn(&mut self, fingerprint: &str, recipe_name: &str, step_index: usize) {
        self.learned.insert(fingerprint.to_string(), LearnedStep {
            recipe_name: recipe_name.to_string(),
            step_index,
        });
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
    let recipes = builtin_recipes();

    if let Some(learned) = kb.recall(fingerprint) {
        if let Some(recipe) = recipes.iter().find(|r| r.name == learned.recipe_name) {
            if let Some(step) = recipe.steps.get(learned.step_index) {
                if let Ok(out) = execute_step(serial, step, diag_port_hint) {
                    return format!("Used learned method for {}:\n{}", fingerprint, out);
                }
            }
        }
    }

    let mut last_error = String::new();
    for recipe in recipes.iter().filter(|r| r.goal == goal) {
        for (step_index, step) in recipe.steps.iter().enumerate() {
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
                        kb.learn(fingerprint, &recipe.name, step_index);
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


#[cfg(not(test))]
fn get_available_ports() -> Result<Vec<serialport::SerialPortInfo>, serialport::Error> {
    serialport::available_ports()
}

#[cfg(test)]
thread_local! {
    pub static MOCK_AVAILABLE_PORTS: std::cell::RefCell<Option<Result<Vec<serialport::SerialPortInfo>, String>>> = std::cell::RefCell::new(None);
}

#[cfg(test)]
fn get_available_ports() -> Result<Vec<serialport::SerialPortInfo>, serialport::Error> {
    MOCK_AVAILABLE_PORTS.with(|m| {
        if let Some(res) = m.borrow().as_ref() {
            match res {
                Ok(ports) => Ok(ports.clone()),
                Err(e) => Err(serialport::Error::new(serialport::ErrorKind::Unknown, e.clone())),
            }
        } else {
            serialport::available_ports()
        }
    })
}

pub fn autodetect_diag_port() -> Option<String> {
    if let Ok(ports) = get_available_ports() {
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


    struct PortsMockGuard;
    impl Drop for PortsMockGuard {
        fn drop(&mut self) {
            MOCK_AVAILABLE_PORTS.with(|m| *m.borrow_mut() = None);
        }
    }

    #[test]
    fn test_autodetect_diag_port_none_when_error() {
        MOCK_AVAILABLE_PORTS.with(|m| *m.borrow_mut() = Some(Err("mock error".into())));
        let _guard = PortsMockGuard;
        let port = autodetect_diag_port();
        assert_eq!(port, None);
    }

    #[test]
    fn test_autodetect_diag_port_none_when_no_ports() {
        MOCK_AVAILABLE_PORTS.with(|m| *m.borrow_mut() = Some(Ok(vec![])));
        let _guard = PortsMockGuard;
        let port = autodetect_diag_port();
        assert_eq!(port, None);
    }

    #[test]
    fn test_autodetect_diag_port_none_when_no_usb_ports() {
        let port_info = serialport::SerialPortInfo {
            port_name: "COM1".into(),
            port_type: serialport::SerialPortType::Unknown,
        };
        MOCK_AVAILABLE_PORTS.with(|m| *m.borrow_mut() = Some(Ok(vec![port_info])));
        let _guard = PortsMockGuard;
        let port = autodetect_diag_port();
        assert_eq!(port, None);
    }

    #[test]
    fn test_autodetect_diag_port_some_when_usb_port_found() {
        let usb_info = serialport::UsbPortInfo {
            vid: 0x1234,
            pid: 0x5678,
            serial_number: None,
            manufacturer: None,
            product: None,
        };
        let port_info = serialport::SerialPortInfo {
            port_name: "COM1".into(),
            port_type: serialport::SerialPortType::UsbPort(usb_info),
        };
        MOCK_AVAILABLE_PORTS.with(|m| *m.borrow_mut() = Some(Ok(vec![port_info])));
        let _guard = PortsMockGuard;
        let port = autodetect_diag_port();
        assert_eq!(port, Some("COM1".into()));
    }

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

        kb.learn("test_fingerprint", "Test Recipe", 0);

        assert!(db_path.exists());

        let loaded_kb = KnowledgeBase::load();
        assert_eq!(loaded_kb.learned.len(), 1);
        let loaded_step = loaded_kb.recall("test_fingerprint").ok_or("missing step")?;
        assert_eq!(loaded_step.recipe_name, "Test Recipe");
        assert_eq!(loaded_step.step_index, 0);

        Ok(())
    }
}
