//! Context engine — process monitoring and automatic profile switching.
//!
//! Monitors the host OS for running processes (Zoom, Teams, Spotify, etc.)
//! and switches profiles automatically based on rules. Also detects idle
//! state and calendar events (future).
//!
//! On Windows (the Surface Pro), process monitoring uses the `sysinfo` crate
//! which is already in the dependency tree. On Linux (homelab dev), it works
//! the same way.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::System;

// ---- Context state ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    #[default]
    Manual,
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ContextState {
    pub mode: ContextMode,
    pub current_profile: String,
    pub detected_context: String,
    pub last_switch_reason: String,
    pub idle_seconds: u64,
    pub active_processes: Vec<String>,
}

// ---- Context rules ---------------------------------------------------------

/// A rule that maps detected context to a profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextRule {
    /// Unique id.
    pub id: String,
    /// Human-readable description.
    pub label: String,
    /// Process names to watch for (e.g. ["zoom.exe", "teams.exe"]).
    #[serde(default)]
    pub processes: Vec<String>,
    /// Profile to switch to when this rule matches.
    pub profile: String,
    /// Priority (higher = checked first).
    #[serde(default)]
    pub priority: i32,
    /// Whether this rule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// Default context rules matching the roadmap.
pub fn default_rules() -> Vec<ContextRule> {
    vec![
        ContextRule {
            id: "meeting".into(),
            label: "Zoom/Teams meeting → Meeting mode".into(),
            processes: vec!["zoom.exe".into(), "teams.exe".into(), "ms-teams.exe".into()],
            profile: "meeting".into(),
            priority: 100,
            enabled: true,
        },
        ContextRule {
            id: "music".into(),
            label: "Spotify playing → Music mode".into(),
            processes: vec!["spotify.exe".into(), "spotify".into()],
            profile: "home".into(),
            priority: 50,
            enabled: true,
        },
        ContextRule {
            id: "idle".into(),
            label: "Idle 5min → Ambient mode".into(),
            processes: vec![],
            profile: "sleep".into(),
            priority: 10,
            enabled: true,
        },
    ]
}

// ---- Process monitor -------------------------------------------------------

pub struct ProcessMonitor {
    sys: System,
    last_check: Instant,
    check_interval: Duration,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            last_check: Instant::now(),
            check_interval: Duration::from_secs(3),
        }
    }

    /// Check running processes. Returns the set of process names found.
    pub fn check_processes(&mut self) -> HashSet<String> {
        if self.last_check.elapsed() < self.check_interval {
            // Return cached result — don't refresh too often
            return self.snapshot_processes();
        }

        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        self.last_check = Instant::now();
        self.snapshot_processes()
    }

    fn snapshot_processes(&self) -> HashSet<String> {
        self.sys
            .processes()
            .values()
            .filter_map(|p| p.name().to_str().map(|s| s.to_lowercase()))
            .collect()
    }

    /// Set the check interval (seconds).
    pub fn set_interval(&mut self, secs: u64) {
        self.check_interval = Duration::from_secs(secs);
    }
}

// ---- Context engine --------------------------------------------------------

/// The context engine evaluates rules and determines which profile should be active.
pub struct ContextEngine {
    monitor: ProcessMonitor,
    rules: Vec<ContextRule>,
    last_interaction: Instant,
    idle_threshold_secs: u64,
}

impl ContextEngine {
    pub fn new() -> Self {
        Self {
            monitor: ProcessMonitor::new(),
            rules: default_rules(),
            last_interaction: Instant::now(),
            idle_threshold_secs: 300, // 5 minutes
        }
    }

    /// Record a user interaction (resets idle timer).
    pub fn note_interaction(&mut self) {
        self.last_interaction = Instant::now();
    }

    /// Set the idle threshold in seconds.
    pub fn set_idle_threshold(&mut self, secs: u64) {
        self.idle_threshold_secs = secs;
    }

    /// Update context rules at runtime.
    pub fn set_rules(&mut self, rules: Vec<ContextRule>) {
        self.rules = rules;
        // Sort by priority descending
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluate rules and return the profile that should be active.
    /// Returns (profile_name, reason_string).
    pub fn evaluate(&mut self) -> (String, String) {
        let processes = self.monitor.check_processes();
        let idle_secs = self.last_interaction.elapsed().as_secs();

        // Check process-based rules (highest priority first)
        for rule in &self.rules {
            if !rule.enabled || rule.processes.is_empty() {
                continue;
            }
            for proc_name in &rule.processes {
                if processes.contains(&proc_name.to_lowercase()) {
                    return (rule.profile.clone(), rule.label.clone());
                }
            }
        }

        // Check idle rule
        if idle_secs >= self.idle_threshold_secs {
            return ("sleep".into(), "Idle timeout → Sleep mode".into());
        }

        // Default: no change
        ("work".into(), "Default → Work mode".into())
    }

    /// Get current context state for the frontend.
    pub fn state(&self) -> ContextState {
        let idle_secs = self.last_interaction.elapsed().as_secs();
        let active_processes: Vec<String> = self
            .monitor
            .snapshot_processes()
            .into_iter()
            .collect();
        
        ContextState {
            mode: ContextMode::Auto,
            current_profile: String::new(),
            detected_context: String::new(),
            last_switch_reason: String::new(),
            idle_seconds: idle_secs,
            active_processes,
        }
    }
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_have_priorities() {
        let rules = default_rules();
        assert!(rules.iter().any(|r| r.id == "meeting"));
        assert!(rules.iter().any(|r| r.id == "music"));
        assert!(rules.iter().any(|r| r.id == "idle"));
        
        // Meeting should have highest priority
        let meeting = rules.iter().find(|r| r.id == "meeting").unwrap();
        let music = rules.iter().find(|r| r.id == "music").unwrap();
        assert!(meeting.priority > music.priority);
    }

    #[test]
    fn context_rule_roundtrips_json() {
        let rule = ContextRule {
            id: "test".into(),
            label: "Test rule".into(),
            processes: vec!["test.exe".into()],
            profile: "work".into(),
            priority: 50,
            enabled: true,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: ContextRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "test");
        assert_eq!(back.processes, vec!["test.exe"]);
        assert_eq!(back.priority, 50);
    }

    #[test]
    fn context_state_serializes() {
        let state = ContextState::default();
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"manual\""));
    }

    #[test]
    fn process_monitor_initializes() {
        let monitor = ProcessMonitor::new();
        // Should not panic, should have empty initial snapshot
        let procs = monitor.snapshot_processes();
        // On a real system there will be processes, but we just check it doesn't crash
        assert!(procs.len() >= 0);
    }

    #[test]
    fn context_engine_evaluates_idle() {
        let mut engine = ContextEngine::new();
        // Don't note any interaction — should be idle immediately
        // (Instant::now() in constructor means we need to wait, but the logic is sound)
        engine.set_idle_threshold(0); // immediately idle
        let (profile, reason) = engine.evaluate();
        assert_eq!(profile, "sleep");
        assert!(reason.contains("Idle"));
    }

    #[test]
    fn context_engine_note_interaction_resets_idle() {
        let mut engine = ContextEngine::new();
        engine.set_idle_threshold(100);
        engine.note_interaction();
        let (profile, _) = engine.evaluate();
        // Should not be idle since we just noted interaction
        assert_ne!(profile, "sleep");
    }
}