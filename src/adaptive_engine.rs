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

fn kb_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".foem").join("learned_methods.json");
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile)
            .join(".foem")
            .join("learned_methods.json");
    }
    PathBuf::from(".foem/learned_methods.json")
}

pub fn fingerprint(model: &str, release: &str, platform: &str) -> String {
    format!(
        "{}|{}|{}",
        model.trim(),
        release.trim(),
        platform.trim()
    )
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
                    if step
                        .success_markers
                        .iter()
                        .any(|m| out.contains(m))
                        || step.success_markers.is_empty()
                    {
                        kb.learn(fingerprint, step.clone());
                        return format!("{} succeeded via {}:\n{}", goal_string(&goal), recipe.name, out);
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
    match step.kind {
        StepKind::AdbShell => exec::run("adb", &["-s", serial, "shell", &step.payload], "ADB shell"),
        StepKind::Fastboot => exec::run("fastboot", &["-s", serial, &step.payload], "Fastboot"),
        StepKind::AtCommand => {
            let port_name = diag_port_hint
                .or_else(|| autodetect_diag_port().as_deref())
                .ok_or_else(|| "No diagnostic port available for AT command".to_string())?;
            let mut port = open_diag_port(port_name)?;
            send_at_command(&mut port, &step.payload)
        }
        StepKind::RawDiag => {
            let port_name = diag_port_hint
                .or_else(|| autodetect_diag_port().as_deref())
                .ok_or_else(|| "No diagnostic port available for DIAG command".to_string())?;
            let mut port = open_diag_port(port_name)?;
            let bytes = hex::decode(step.payload.replace([' ', '\n', '\r'], ""))
                .map_err(|e| format!("Hex decode failed: {}", e))?;
            crate::features::repair::send_diag_bytes(&mut *port, &bytes)
        }
    }
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
            }],
        },
    ]
}

fn autodetect_diag_port() -> Option<String> {
    if let Ok(ports) = serialport::available_ports() {
        for p in ports {
            if matches!(p.port_type, serialport::SerialPortType::UsbPort(_)) {
                return Some(p.port_name);
            }
        }
    }
    None
}
