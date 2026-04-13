//! AI Assistant module powering the in-app "Agentic Copilot".
//! Provides provider-agnostic chat requests, attachment ingestion, and
//! safety-focused command extraction.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Supported API providers (OpenAI compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenRouter,
    OpenAI,
    Gemini,
    Local,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::OpenRouter => "OpenRouter",
            Provider::OpenAI => "OpenAI",
            Provider::Gemini => "Gemini",
            Provider::Local => "Local / Custom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiSettings {
    pub provider: Provider,
    pub api_key: String,
    pub custom_endpoint: String,
    pub model: String,
    pub vision_enabled: bool,
    pub temperature: f32,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: Provider::OpenRouter,
            api_key: String::new(),
            custom_endpoint: "http://localhost:11434/v1/chat/completions".into(),
            model: "gpt-4.1".into(),
            vision_enabled: true,
            temperature: 0.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentKind {
    Text,
    Image,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub kind: AttachmentKind,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(default)]
    pub is_command: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TelemetrySnapshot {
    pub active_panel: String,
    pub recent_actions: Vec<String>,
    pub device_summary: String,
}

#[derive(Default, Debug, Clone)]
pub struct AiAssistantState {
    pub history: Vec<ChatMessage>,
    pub input: String,
    pub last_error: Option<String>,
    pub pending_commands: Vec<String>,
    pub attachments: Vec<Attachment>,
    pub last_action_target: Option<String>,
}

impl AiAssistantState {
    pub fn system_prompt(&self, telemetry: &TelemetrySnapshot) -> String {
        let mut prompt = String::from(
            "You are FOEM's built-in phone repair copilot. Provide concise, safe steps.\n\
             Use JSON tool outputs only when you intend to navigate UI: {\"action\":\"navigate_ui\",\"target\":\"PanelName\"}.\n\
             NEVER auto-execute commands. Output shell commands inside fenced blocks so the UI can present Execute buttons.",
        );
        prompt.push_str(&format!(
            "\nCurrent panel: {}\nRecent actions: {}\nDevice: {}",
            telemetry.active_panel,
            telemetry
                .recent_actions
                .iter()
                .rev()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | "),
            telemetry.device_summary
        ));
        prompt
    }

    pub fn add_attachment_from_path(&mut self, path: &str) -> Result<(), String> {
        if path.trim().is_empty() {
            return Err("Attachment path is empty.".into());
        }
        let p = Path::new(path);
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("attachment")
            .to_string();
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp");
        let bytes = fs::read(p).map_err(|e| format!("Unable to read file: {}", e))?;
        if is_image {
            let encoded = STANDARD.encode(&bytes);
            let mime = match ext.as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "bmp" => "image/bmp",
                _ => "image/*",
            };
            self.attachments.push(Attachment {
                name,
                mime: mime.to_string(),
                kind: AttachmentKind::Image,
                content: encoded,
            });
        } else {
            let text = String::from_utf8(bytes)
                .map_err(|e| format!("Attachment is not valid UTF-8 text: {}", e))?;
            self.attachments.push(Attachment {
                name,
                mime: "text/plain".into(),
                kind: AttachmentKind::Text,
                content: text,
            });
        }
        Ok(())
    }

    pub fn push_user_message(&mut self, content: String) {
        self.history.push(ChatMessage {
            role: ChatRole::User,
            content,
            is_command: false,
        });
    }

    pub fn push_assistant_message(&mut self, content: String) {
        self.history.push(ChatMessage {
            role: ChatRole::Assistant,
            content,
            is_command: false,
        });
    }

    pub fn extract_commands(&mut self, text: &str) {
        let mut commands = Vec::new();
        let mut in_block = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_block = !in_block;
                continue;
            }
            if in_block && !trimmed.is_empty() {
                commands.push(trimmed.to_string());
            } else if trimmed.starts_with("adb ")
                || trimmed.starts_with("fastboot ")
                || trimmed.starts_with("python ")
            {
                commands.push(trimmed.to_string());
            }
        }
        self.pending_commands = commands;
    }

    pub fn detect_navigation_action(&mut self, text: &str) {
        // Direct JSON block
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
            if val
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                == "navigate_ui"
            {
                if let Some(target) = val.get("target").and_then(|v| v.as_str()) {
                    self.last_action_target = Some(target.to_string());
                    return;
                }
            }
        }

        // Fallback: search for first JSON-like block within text
        for line in text.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if val
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    == "navigate_ui"
                {
                    if let Some(target) = val.get("target").and_then(|v| v.as_str()) {
                        self.last_action_target = Some(target.to_string());
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Serialize)]
struct ChatRequestMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<serde_json::Value>,
}

fn endpoint(settings: &AiSettings) -> String {
    match settings.provider {
        Provider::OpenRouter => "https://openrouter.ai/api/v1/chat/completions".into(),
        Provider::OpenAI => "https://api.openai.com/v1/chat/completions".into(),
        Provider::Gemini => {
            // OpenAI-compatible bridge endpoint for Gemini (v1beta)
            format!(
                "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions?key={}",
                settings.api_key
            )
        }
        Provider::Local => settings.custom_endpoint.clone(),
    }
}

fn auth_header(settings: &AiSettings) -> Option<(String, String)> {
    match settings.provider {
        Provider::OpenRouter | Provider::OpenAI => Some((
            "Authorization".into(),
            format!("Bearer {}", settings.api_key),
        )),
        Provider::Gemini => None, // key is in query param above
        Provider::Local => {
            if settings.api_key.is_empty() {
                None
            } else {
                Some((
                    "Authorization".into(),
                    format!("Bearer {}", settings.api_key),
                ))
            }
        }
    }
}

pub fn send_chat(
    state: &mut AiAssistantState,
    settings: &AiSettings,
    telemetry: TelemetrySnapshot,
) -> Result<String, String> {
    let sys = state.system_prompt(&telemetry);
    let mut messages = Vec::new();
    messages.push(ChatRequestMessage {
        role: "system".into(),
        content: Some(json!([{"type": "text", "text": sys}])),
    });

    for msg in &state.history {
        messages.push(ChatRequestMessage {
            role: match msg.role {
                ChatRole::System => "system".into(),
                ChatRole::User => "user".into(),
                ChatRole::Assistant => "assistant".into(),
            },
            content: Some(json!([{"type": "text", "text": &msg.content}])),
        });
    }

    // Combine current input with attachments
    let mut content_blocks = vec![json!({"type": "text", "text": &state.input})];
    for att in &state.attachments {
        match att.kind {
            AttachmentKind::Text => {
                content_blocks.push(json!({
                    "type": "text",
                    "text": format!("Attachment: {}\n{}", att.name, att.content)
                }));
            }
            AttachmentKind::Image => {
                if settings.vision_enabled {
                    content_blocks.push(json!({
                        "type": "image_url",
                        "image_url": { "url": format!("data:{};base64,{}", att.mime, att.content) }
                    }));
                } else {
                    content_blocks.push(json!({
                        "type": "text",
                        "text": format!("(Vision disabled) Attached image: {}", att.name)
                    }));
                }
            }
        }
    }

    messages.push(ChatRequestMessage {
        role: "user".into(),
        content: Some(json!(content_blocks)),
    });

    let req_body = json!({
        "model": settings.model.clone(),
        "messages": messages,
        "temperature": settings.temperature
    });

    let url = endpoint(settings);
    let mut request = ureq::post(&url).set("Content-Type", "application/json");
    if let Some((header, value)) = auth_header(settings) {
        request = request.set(&header, &value);
    }

    let response = request
        .send_json(req_body)
        .map_err(|e| format!("AI request failed: {}", e))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("AI response parse failed: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string();

    state.push_assistant_message(content.clone());
    state.extract_commands(&content);
    state.detect_navigation_action(&content);
    state.attachments.clear();
    state.input.clear();
    Ok(content)
}
