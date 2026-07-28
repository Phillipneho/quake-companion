//! OpenClaw panel — voice + text chat interface over Tailscale.
//!
//! Flow: Hold knob → record mic → send audio to homelab → Whisper STT →
//! LLM response streams back → rendered as conversation history.
//!
//! The audio is sent to the homelab's HTTP endpoint, which runs the Whisper
//! API transcription script, then forwards the text to OpenClaw for a response.

use serde::{Deserialize, Serialize};
// ---- Chat message types ----------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "role", content = "content")]
pub enum ChatMessage {
    User(String),
    Assistant(String),
    System(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
    pub started_at: String,
}

// ---- Recording state -------------------------------------------------------

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
    Thinking,
    Responding,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---- Transcription --------------------------------------------------------

/// Transcribe audio using the OpenAI Whisper API via the homelab's script.
/// The audio bytes (WAV/webm) are written to a temp file, then the
/// transcribe.sh script is run.
pub async fn transcribe_audio(audio_data: &[u8], format: &str) -> Result<String, String> {
    use std::io::Write;
    
    // Write audio to temp file
    let ext = match format {
        "wav" => "wav",
        "webm" => "webm",
        "mp3" => "mp3",
        "m4a" => "m4a",
        _ => "wav",
    };
    
    let tmp_path = format!("/tmp/quake-recording-{}.{}", 
        chrono::Utc::now().timestamp_millis(), ext);
    
    {
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("failed to create temp file: {e}"))?;
        file.write_all(audio_data)
            .map_err(|e| format!("failed to write audio: {e}"))?;
    }

    // Call the Whisper transcription script
    let script_path = dirs::home_dir()
        .map(|h| h.join(".nvm/versions/node/v22.22.3/lib/node_modules/openclaw/skills/openai-whisper-api/scripts/transcribe.sh"))
        .ok_or("cannot find home dir")?;

    let output = tokio::process::Command::new("bash")
        .arg(&script_path)
        .arg(&tmp_path)
        .arg("--model")
        .arg("whisper-1")
        .env("OPENAI_API_KEY", std::env::var("OPENAI_API_KEY").unwrap_or_default())
        .output()
        .await
        .map_err(|e| format!("whisper script failed: {e}"))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&tmp_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("whisper transcription failed: {stderr}"));
    }

    // The script writes to <input>.txt — read that
    let transcript_path = format!("{}.txt", tmp_path);
    let transcript = std::fs::read_to_string(&transcript_path)
        .map_err(|e| format!("failed to read transcript: {e}"))?;
    
    let _ = std::fs::remove_file(&transcript_path);

    Ok(transcript.trim().to_string())
}

// ---- LLM response (simplified) --------------------------------------------
// In the full implementation, this would stream from OpenClaw over Tailscale.
// For now, we use a simple HTTP call to the OpenClaw gateway or a direct LLM API.

/// Send text to the LLM and get a response. This is a placeholder that
/// will be replaced with a streaming Tailscale connection to OpenClaw.
pub async fn llm_respond(messages: &[ChatMessage]) -> Result<String, String> {
    // Build a simple prompt from the conversation
    let _prompt = messages.iter()
        .map(|m| match m {
            ChatMessage::User(text) => format!("User: {text}"),
            ChatMessage::Assistant(text) => format!("Assistant: {text}"),
            ChatMessage::System(text) => format!("System: {text}"),
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // TODO: Replace with streaming call to OpenClaw over Tailscale.
    // For now, return a placeholder so the widget works end-to-end.
    Ok(format!("I received your message. The full LLM streaming integration will be wired up next. Your message was: {}", 
        messages.iter().rev().find_map(|m| match m {
            ChatMessage::User(text) => Some(text.clone()),
            _ => None,
        }).unwrap_or_default()))
}

// ---- Conversation store ----------------------------------------------------

use std::sync::Mutex;

pub static CONVERSATION: std::sync::LazyLock<Mutex<Conversation>> =
    std::sync::LazyLock::new(|| Mutex::new(Conversation {
        messages: vec![ChatMessage::System(
            "You are Leo, a personal AI assistant running on the QUAKE Companion display panel. \
             You are concise, helpful, and have personality. The user is Phillip."
                .to_string(),
        )],
        started_at: chrono::Utc::now().to_rfc3339(),
    }));

/// Add a message to the conversation.
pub fn add_message(msg: ChatMessage) {
    if let Ok(mut conv) = CONVERSATION.lock() {
        conv.messages.push(msg);
    }
}

/// Get all messages from the conversation.
pub fn get_messages() -> Vec<ChatMessage> {
    CONVERSATION.lock()
        .map(|c| c.messages.clone())
        .unwrap_or_default()
}

/// Clear the conversation (keep only the system message).
pub fn clear_conversation() {
    if let Ok(mut conv) = CONVERSATION.lock() {
        conv.messages.truncate(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_add_and_get() {
        // Use a fresh conversation for testing
        clear_conversation();
        add_message(ChatMessage::User("Hello".into()));
        add_message(ChatMessage::Assistant("Hi there".into()));
        
        let msgs = get_messages();
        assert!(msgs.len() >= 3); // system + user + assistant
    }

    #[test]
    fn clear_keeps_system_message() {
        clear_conversation();
        add_message(ChatMessage::User("test".into()));
        clear_conversation();
        
        let msgs = get_messages();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], ChatMessage::System(_)));
    }

    #[test]
    fn recording_state_serializes() {
        let s = serde_json::to_string(&RecordingState::Recording).unwrap();
        assert_eq!(s, "\"recording\"");
        
        let idle: RecordingState = serde_json::from_str("\"idle\"").unwrap();
        assert_eq!(idle, RecordingState::Idle);
    }

    #[test]
    fn chat_message_serializes() {
        let msg = ChatMessage::User("Hello world".into());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"User\""));
        assert!(json.contains("\"Hello world\""));
    }
}