//! Widget SDK — manifest definitions, widget registry, and plugin loading.
//!
//! Widgets are self-contained packages living in `{config_dir}/quake-companion/widgets/`.
//! Each widget directory contains:
//!
//! - `manifest.json` — metadata, options schema, zone requirements, data needs
//! - `index.html` — the widget's UI (loaded in an iframe or shadow DOM)
//! - optional `icon.svg` — icon for the page selector
//!
//! Built-in widgets (flip-clock, weather, system-monitor, etc.) are registered
//! automatically and don't require files on disk.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---- Widget manifest -------------------------------------------------------

/// The manifest describing a widget's capabilities and requirements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetManifest {
    /// Unique widget id (e.g. "flip-clock", "weather", "openclaw-chat").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semantic version string (e.g. "1.0.0").
    pub version: String,
    /// Author / source.
    #[serde(default)]
    pub author: String,
    /// Minimum zone width in pixels required to render this widget.
    #[serde(default = "default_min_width")]
    pub min_width: u32,
    /// Preferred zone width in pixels.
    #[serde(default = "default_pref_width")]
    pub preferred_width: u32,
    /// Whether this widget can span the full 1920px display.
    #[serde(default = "default_true")]
    pub can_fullscreen: bool,
    /// Data sources this widget needs (e.g. ["weather", "system_stats"]).
    #[serde(default)]
    pub data_sources: Vec<String>,
    /// Knob actions this widget supports (e.g. ["scroll", "next", "prev"]).
    #[serde(default)]
    pub knob_actions: Vec<String>,
    /// Whether the widget needs network access (for iframe widgets).
    #[serde(default)]
    pub needs_network: bool,
    /// Whether this is a built-in widget (not loaded from disk).
    #[serde(default)]
    pub builtin: bool,
    /// Default options (JSON object, interpreted by the widget frontend).
    #[serde(default)]
    pub default_options: serde_json::Value,
}

fn default_min_width() -> u32 { 320 }
fn default_pref_width() -> u32 { 640 }
fn default_true() -> bool { true }

// ---- Widget registry -------------------------------------------------------

/// Registry of all known widgets (built-in + loaded from disk).
#[derive(Clone, Debug, Default)]
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetManifest>,
}

impl WidgetRegistry {
    /// Create a new registry and populate it with built-in widgets + on-disk widgets.
    pub fn load() -> io::Result<Self> {
        let mut registry = Self::default();

        // Register built-in widgets.
        for manifest in builtin_widgets() {
            registry.register(manifest);
        }

        // Load widgets from disk.
        if let Some(widgets_dir) = widgets_dir() {
            if widgets_dir.exists() {
                registry.load_from_dir(&widgets_dir)?;
            }
        }

        Ok(registry)
    }

    /// Register a widget manifest.
    pub fn register(&mut self, manifest: WidgetManifest) {
        self.widgets.insert(manifest.id.clone(), manifest);
    }

    /// Get a widget manifest by id.
    pub fn get(&self, id: &str) -> Option<&WidgetManifest> {
        self.widgets.get(id)
    }

    /// List all registered widgets.
    pub fn list(&self) -> Vec<&WidgetManifest> {
        self.widgets.values().collect()
    }

    /// List widget ids only.
    pub fn ids(&self) -> Vec<String> {
        self.widgets.keys().cloned().collect()
    }

    /// Check if a widget is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.widgets.contains_key(id)
    }

    /// Load widgets from a directory. Each subdirectory should contain a manifest.json.
    fn load_from_dir(&mut self, dir: &PathBuf) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let manifest_path = entry.path().join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let data = fs::read_to_string(&manifest_path)?;
            let manifest: WidgetManifest = match serde_json::from_str(&data) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("Failed to parse widget manifest at {:?}: {}", manifest_path, e);
                    continue;
                }
            };

            // Don't allow disk widgets to override built-ins with the same id.
            if self.widgets.contains_key(&manifest.id) && self.widgets[&manifest.id].builtin {
                log::warn!("Widget '{}' from disk conflicts with built-in, skipping", manifest.id);
                continue;
            }

            self.register(manifest);
        }
        Ok(())
    }
}

// ---- Path helpers ----------------------------------------------------------

/// Return the widgets directory path: `{config_dir}/quake-companion/widgets/`.
pub fn widgets_dir() -> Option<PathBuf> {
    crate::config::config_dir().map(|d| d.join("widgets"))
}

// ---- Built-in widget definitions ------------------------------------------

fn builtin_widgets() -> Vec<WidgetManifest> {
    vec![
        WidgetManifest {
            id: "flip-clock".into(),
            name: "Flip Clock".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: true,
            data_sources: vec![],
            knob_actions: vec!["toggle-format".into(), "toggle-seconds".into()],
            needs_network: false,
            builtin: true,
            default_options: serde_json::json!({
                "show_seconds": false,
                "format_24h": true
            }),
        },
        WidgetManifest {
            id: "weather".into(),
            name: "Weather".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec!["weather".into()],
            knob_actions: vec!["scroll-forecast".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({
                "location": "Brisbane",
                "units": "metric"
            }),
        },
        WidgetManifest {
            id: "system-monitor".into(),
            name: "System Monitor".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec!["system_stats".into()],
            knob_actions: vec!["cycle-metric".into(), "expand".into()],
            needs_network: false,
            builtin: true,
            default_options: serde_json::json!({
                "show_cpu": true,
                "show_ram": true,
                "show_disk": true,
                "show_network": true
            }),
        },
        WidgetManifest {
            id: "now-playing".into(),
            name: "Now Playing".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec!["spotify".into()],
            knob_actions: vec!["volume".into(), "play-pause".into(), "next".into(), "prev".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({}),
        },
        WidgetManifest {
            id: "notifications".into(),
            name: "Notifications".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec!["discord".into(), "calendar".into(), "github".into()],
            knob_actions: vec!["scroll".into(), "open".into(), "dismiss".into(), "snooze".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({}),
        },
        WidgetManifest {
            id: "openclaw-chat".into(),
            name: "OpenClaw Panel".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 640,
            preferred_width: 1920,
            can_fullscreen: true,
            data_sources: vec!["openclaw".into()],
            knob_actions: vec!["record".into(), "send".into(), "scroll".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({
                "voice_mode": false
            }),
        },
        WidgetManifest {
            id: "web-dashboard".into(),
            name: "Web Dashboard".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 640,
            preferred_width: 1920,
            can_fullscreen: true,
            data_sources: vec![],
            knob_actions: vec!["scroll".into(), "click".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({
                "url": ""
            }),
        },
        WidgetManifest {
            id: "pomodoro".into(),
            name: "Pomodoro Timer".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec![],
            knob_actions: vec!["adjust".into(), "pause".into()],
            needs_network: false,
            builtin: true,
            default_options: serde_json::json!({
                "work_minutes": 25,
                "break_minutes": 5
            }),
        },
        WidgetManifest {
            id: "calendar".into(),
            name: "Calendar".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: true,
            data_sources: vec!["calendar".into()],
            knob_actions: vec!["scroll-days".into(), "join-meeting".into()],
            needs_network: true,
            builtin: true,
            default_options: serde_json::json!({}),
        },
        WidgetManifest {
            id: "daily-stoic".into(),
            name: "Daily Stoic".into(),
            version: "1.0.0".into(),
            author: "QUAKE Companion".into(),
            min_width: 320,
            preferred_width: 640,
            can_fullscreen: false,
            data_sources: vec![],
            knob_actions: vec!["next-quote".into(), "save-favourite".into()],
            needs_network: false,
            builtin: true,
            default_options: serde_json::json!({}),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_builtins() {
        let registry = WidgetRegistry::load().unwrap();
        assert!(registry.contains("flip-clock"));
        assert!(registry.contains("weather"));
        assert!(registry.contains("system-monitor"));
        assert!(registry.contains("openclaw-chat"));
    }

    #[test]
    fn builtin_manifest_has_required_fields() {
        let registry = WidgetRegistry::load().unwrap();
        let clock = registry.get("flip-clock").unwrap();
        assert!(!clock.id.is_empty());
        assert!(!clock.name.is_empty());
        assert!(!clock.version.is_empty());
        assert!(clock.builtin);
        assert!(clock.min_width > 0);
        assert!(clock.preferred_width >= clock.min_width);
    }

    #[test]
    fn manifest_roundtrips_json() {
        let manifest = builtin_widgets().into_iter().next().unwrap();
        let json = serde_json::to_string(&manifest).unwrap();
        let back: WidgetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest.id, back.id);
        assert_eq!(manifest.name, back.name);
        assert_eq!(manifest.version, back.version);
    }

    #[test]
    fn registry_list_returns_all() {
        let registry = WidgetRegistry::load().unwrap();
        let list = registry.list();
        assert!(list.len() >= 10); // at least 10 builtins
    }

    #[test]
    fn fullscreen_widgets_have_large_preferred_width() {
        let registry = WidgetRegistry::load().unwrap();
        let chat = registry.get("openclaw-chat").unwrap();
        assert!(chat.can_fullscreen);
        assert_eq!(chat.preferred_width, 1920);
    }

    #[test]
    fn weather_widget_needs_network() {
        let registry = WidgetRegistry::load().unwrap();
        let weather = registry.get("weather").unwrap();
        assert!(weather.needs_network);
        assert!(weather.data_sources.contains(&"weather".to_string()));
    }
}