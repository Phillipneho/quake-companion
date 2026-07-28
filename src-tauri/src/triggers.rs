//! Trigger / automation engine — "When X → do Y" rules.
//!
//! Triggers fire on: process events, calendar events, webhooks, system events,
//! time-based schedules. Actions: switch profile, switch page, set ring colour,
//! send notification, run command.
//!
//! The engine evaluates triggers periodically and fires matching actions.

use std::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::via::RgbEffect;

// ---- Trigger types ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Process,
    Calendar,
    Webhook,
    System,
    Time,
    Manual,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SwitchProfile,
    SwitchPage,
    SetRing,
    Notify,
    RunCommand,
}

/// A trigger condition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub label: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub trigger_type: TriggerType,
    /// Condition-specific config (process name, calendar keyword, cron expr, etc.)
    #[serde(default)]
    pub condition: serde_json::Value,
    pub action: ActionType,
    /// Action-specific config (profile name, page index, ring effect, etc.)
    #[serde(default)]
    pub action_data: serde_json::Value,
}

fn default_true() -> bool { true }

// ---- Ring status presets ---------------------------------------------------

/// Ambient ring status presets tied to the trigger engine.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RingStatus {
    Healthy,     // green
    Error,       // red
    Unread,      // amber
    MicLive,     // breathing white
    Building,    // blue
    Focusing,    // rainbow
    Off,         // sleep / off
}

impl RingStatus {
    /// Map a ring status to VIA effect + color.
    pub fn to_via(&self) -> (RgbEffect, [u8; 2], u8) {
        match self {
            Self::Healthy => (RgbEffect::Plain, [120, 255], 128),  // green
            Self::Error => (RgbEffect::Plain, [0, 255], 200),       // red
            Self::Unread => (RgbEffect::Plain, [30, 255], 128),     // amber
            Self::MicLive => (RgbEffect::Breathe, [170, 255], 128), // breathing white-ish
            Self::Building => (RgbEffect::Plain, [210, 255], 100), // blue
            Self::Focusing => (RgbEffect::Rainbow, [0, 255], 128),  // rainbow
            Self::Off => (RgbEffect::Off, [0, 0], 0),               // off
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Error => "Error",
            Self::Unread => "Unread",
            Self::MicLive => "Mic Live",
            Self::Building => "Building",
            Self::Focusing => "Focusing",
            Self::Off => "Off",
        }
    }
}

// ---- Trigger engine state --------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TriggerEngineState {
    pub current_ring_status: Option<RingStatus>,
    pub last_fired_trigger: Option<String>,
    pub active_triggers: usize,
}

// ---- Trigger engine --------------------------------------------------------

pub struct TriggerEngine {
    triggers: Vec<Trigger>,
    state: TriggerEngineState,
}

impl TriggerEngine {
    pub fn new() -> Self {
        Self {
            triggers: default_triggers(),
            state: TriggerEngineState::default(),
        }
    }

    pub fn set_triggers(&mut self, triggers: Vec<Trigger>) {
        self.triggers = triggers;
    }

    pub fn triggers(&self) -> &[Trigger] {
        &self.triggers
    }

    pub fn state(&self) -> &TriggerEngineState {
        &self.state
    }

    /// Set the ring status manually (overrides trigger-based status).
    pub fn set_ring_status(&mut self, status: RingStatus) {
        self.state.current_ring_status = Some(status);
    }

    /// Fire a trigger by id.
    pub fn fire(&mut self, trigger_id: &str) -> Option<&Trigger> {
        if let Some(trigger) = self.triggers.iter().find(|t| t.id == trigger_id) {
            self.state.last_fired_trigger = Some(trigger_id.to_string());
            return Some(trigger);
        }
        None
    }

    /// Evaluate time-based triggers and return any that should fire.
    pub fn evaluate_time_triggers(&self) -> Vec<&Trigger> {
        let now = chrono::Utc::now();
        self.triggers.iter().filter(|t| {
            t.enabled && t.trigger_type == TriggerType::Time
        }).filter(|t| {
            // Simple hour-based check: condition has "hour" field
            if let Some(hour) = t.condition.get("hour").and_then(|v| v.as_u64()) {
                now.format("%H").to_string().parse::<u64>().unwrap_or(0) == hour
            } else {
                false
            }
        }).collect()
    }
}

impl Default for TriggerEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Default triggers matching the roadmap.
fn default_triggers() -> Vec<Trigger> {
    vec![
        Trigger {
            id: "ci-fail".into(),
            label: "CI fails → switch page + ring red".into(),
            enabled: true,
            trigger_type: TriggerType::Webhook,
            condition: serde_json::json!({"event": "ci_failed"}),
            action: ActionType::SetRing,
            action_data: serde_json::json!({"ring": "error"}),
        },
        Trigger {
            id: "zoom-starts".into(),
            label: "Zoom starts → meeting mode".into(),
            enabled: true,
            trigger_type: TriggerType::Process,
            condition: serde_json::json!({"process": "zoom.exe"}),
            action: ActionType::SwitchProfile,
            action_data: serde_json::json!({"profile": "meeting"}),
        },
        Trigger {
            id: "calendar-soon".into(),
            label: "Calendar event in 5min → agenda view".into(),
            enabled: true,
            trigger_type: TriggerType::Calendar,
            condition: serde_json::json!({"minutes_before": 5}),
            action: ActionType::SwitchPage,
            action_data: serde_json::json!({"page": "calendar"}),
        },
        Trigger {
            id: "container-down".into(),
            label: "Container down → notification + ring amber".into(),
            enabled: true,
            trigger_type: TriggerType::System,
            condition: serde_json::json!({"check": "docker_container"}),
            action: ActionType::Notify,
            action_data: serde_json::json!({"message": "Container is down", "ring": "unread"}),
        },
        Trigger {
            id: "sleep-time".into(),
            label: "10 PM → sleep mode, ring off".into(),
            enabled: true,
            trigger_type: TriggerType::Time,
            condition: serde_json::json!({"hour": 22}),
            action: ActionType::SwitchProfile,
            action_data: serde_json::json!({"profile": "sleep", "ring": "off"}),
        },
    ]
}

pub static TRIGGER_ENGINE: std::sync::LazyLock<Mutex<TriggerEngine>> =
    std::sync::LazyLock::new(|| Mutex::new(TriggerEngine::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_triggers_exist() {
        let engine = TriggerEngine::new();
        assert!(engine.triggers().iter().any(|t| t.id == "ci-fail"));
        assert!(engine.triggers().iter().any(|t| t.id == "zoom-starts"));
        assert!(engine.triggers().iter().any(|t| t.id == "sleep-time"));
    }

    #[test]
    fn ring_status_maps_to_via() {
        let (effect, color, brightness) = RingStatus::Healthy.to_via();
        assert_eq!(effect, RgbEffect::Plain);
        assert_eq!(brightness, 128);

        let (effect, _, _) = RingStatus::Off.to_via();
        assert_eq!(effect, RgbEffect::Off);

        let (effect, _, _) = RingStatus::Focusing.to_via();
        assert_eq!(effect, RgbEffect::Rainbow);
    }

    #[test]
    fn trigger_roundtrips_json() {
        let trigger = Trigger {
            id: "test".into(),
            label: "Test trigger".into(),
            enabled: true,
            trigger_type: TriggerType::Process,
            condition: serde_json::json!({"process": "test.exe"}),
            action: ActionType::SwitchProfile,
            action_data: serde_json::json!({"profile": "work"}),
        };
        let json = serde_json::to_string(&trigger).unwrap();
        let back: Trigger = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "test");
        assert_eq!(back.trigger_type, TriggerType::Process);
        assert_eq!(back.action, ActionType::SwitchProfile);
    }

    #[test]
    fn fire_trigger_sets_last_fired() {
        let mut engine = TriggerEngine::new();
        engine.fire("zoom-starts");
        assert_eq!(engine.state().last_fired_trigger, Some("zoom-starts".to_string()));
    }

    #[test]
    fn set_ring_status_updates_state() {
        let mut engine = TriggerEngine::new();
        engine.set_ring_status(RingStatus::Error);
        assert_eq!(engine.state().current_ring_status, Some(RingStatus::Error));
    }

    #[test]
    fn ring_status_labels() {
        assert_eq!(RingStatus::Healthy.label(), "Healthy");
        assert_eq!(RingStatus::Error.label(), "Error");
        assert_eq!(RingStatus::Off.label(), "Off");
        assert_eq!(RingStatus::MicLive.label(), "Mic Live");
    }
}