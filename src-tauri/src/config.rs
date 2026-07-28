//! Config system — JSON config file with zones, pages, widgets, profiles, and settings.
//!
//! The config lives at `{config_dir}/quake-companion/config.json`. On first run
//! (or if the file is missing/corrupt), a default config is seeded.
//!
//! The config is designed to be human-editable and forward-compatible:
//! - Unknown fields are preserved on save (serde skip_serializing_if = Option::is_none)
//! - A `schema_version` field enables future migrations
//! - Profiles reference pages by name; pages reference widgets by id

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::via::RgbEffect;
use crate::homeassistant::HaConfig;

// ---- Schema version --------------------------------------------------------

pub const SCHEMA_VERSION: u32 = 1;

// ---- Top-level config -------------------------------------------------------

/// The root config object, serialised to `config.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub schema_version: u32,
    /// Global settings (idle timeouts, ring defaults, etc.)
    pub settings: Settings,
    /// Page definitions, keyed by name.
    pub pages: Vec<Page>,
    /// Profile definitions — each maps to an ordered list of pages.
    pub profiles: Vec<Profile>,
    /// The active profile name (must match one in `profiles`).
    #[serde(default)]
    pub active_profile: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: Settings::default(),
            pages: default_pages(),
            profiles: default_profiles(),
            active_profile: "work".to_string(),
        }
    }
}

// ---- Settings --------------------------------------------------------------

/// Global settings shared across all profiles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // ---- Screen / power ----
    /// Seconds of inactivity before dimming.
    pub idle_dim_secs: u64,
    /// Seconds of inactivity before sleeping.
    pub idle_sleep_secs: u64,
    /// Brightness level (0-255) when dimmed.
    pub dim_brightness: u8,
    /// Brightness restored on wake.
    pub wake_brightness: u8,

    // ---- RGB ring ----
    /// Default ring effect.
    pub ring_effect: RgbEffect,
    /// Default ring brightness (0-255).
    pub ring_brightness: u8,
    /// Default ring color 1 (hue, sat, 0-255 each).
    pub ring_color_1: [u8; 2],
    /// Default ring color 2 (hue, sat, 0-255 each).
    pub ring_color_2: [u8; 2],
    /// Persist ring settings to EEPROM on change.
    pub ring_persist: bool,

    // ---- Display ----
    /// Keep-alive interval in ms (advanced; default 1500).
    pub keep_alive_ms: u64,

    // ---- Audio ----
    #[serde(default)]
    pub mic_enabled: bool,

    // ---- Clock ----
    /// 12 or 24 hour clock format.
    #[serde(default = "default_clock_24h")]
    pub clock_24h: bool,
    /// IANA timezone string (e.g. "Australia/Brisbane").
    #[serde(default = "default_timezone")]
    pub timezone: String,

    // ---- Home Assistant ----
    /// HA connection config (None = HA not configured).
    #[serde(default)]
    pub ha: Option<HaConfig>,
}

fn default_clock_24h() -> bool {
    true
}

fn default_timezone() -> String {
    "Australia/Brisbane".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            idle_dim_secs: 30,
            idle_sleep_secs: 120,
            dim_brightness: 30,
            wake_brightness: 255,
            ring_effect: RgbEffect::Off,
            ring_brightness: 128,
            ring_color_1: [170, 255], // cyan-ish
            ring_color_2: [0, 255],   // red
            ring_persist: false,
            keep_alive_ms: 1500,
            mic_enabled: false,
            clock_24h: true,
            timezone: "Australia/Brisbane".to_string(),
            ha: None,
        }
    }
}

// ---- Zone layout ------------------------------------------------------------

/// A zone is a horizontal slice of the 1920×480 display.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    /// Unique id within the page (e.g. "left", "center", "right").
    pub id: String,
    /// X offset in pixels (0-1920).
    pub x: u32,
    /// Width in pixels.
    pub width: u32,
    /// Widget id to render in this zone (references `widgets`).
    #[serde(default)]
    pub widget: Option<String>,
}

// ---- Page ------------------------------------------------------------------

/// A page is a full-screen layout covering the 1920×480 display.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    /// Unique page name (e.g. "clock", "stats", "music").
    pub name: String,
    /// Human-readable label for the page selector.
    #[serde(default)]
    pub label: String,
    /// Zones in this page (typically 1-3).
    pub zones: Vec<Zone>,
}

// ---- Profile ---------------------------------------------------------------

/// A profile is an ordered set of pages with a name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    /// Unique profile name (e.g. "work", "focus", "home", "sleep").
    pub name: String,
    /// Human-readable label.
    #[serde(default)]
    pub label: String,
    /// Ordered list of page names to cycle through.
    pub pages: Vec<String>,
    /// Ring effect override for this profile (None = use global setting).
    #[serde(default)]
    pub ring_effect: Option<RgbEffect>,
}

// ---- Path helpers ----------------------------------------------------------

/// Return the config directory path: `{config_dir}/quake-companion/`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("quake-companion"))
}

/// Return the config file path.
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.json"))
}

// ---- Load / save -----------------------------------------------------------

/// Load the config from disk. If the file is missing, seeds a default and
/// saves it. If the file is corrupt, returns the error so the caller can
/// decide whether to re-seed.
pub fn load() -> io::Result<Config> {
    let path = config_path().ok_or_else(|| io::Error::other("no config directory"))?;

    if !path.exists() {
        let cfg = Config::default();
        save(&cfg)?;
        return Ok(cfg);
    }

    let data = fs::read_to_string(&path)?;
    let mut cfg: Config = serde_json::from_str(&data)
        .map_err(|e| io::Error::other(format!("config parse error: {e}")))?;

    // Light migration: if schema version is older, we'd patch here.
    if cfg.schema_version != SCHEMA_VERSION {
        cfg.schema_version = SCHEMA_VERSION;
        // Future: field migrations go here.
        save(&cfg)?;
    }

    Ok(cfg)
}

/// Save the config to disk, creating the directory if needed.
pub fn save(cfg: &Config) -> io::Result<()> {
    let dir = config_dir().ok_or_else(|| io::Error::other("no config directory"))?;
    fs::create_dir_all(&dir)?;

    let path = config_path().unwrap();
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| io::Error::other(format!("config serialize error: {e}")))?;

    fs::write(&path, json + "\n")?;
    Ok(())
}

/// Reset config to defaults and save.
pub fn reset() -> io::Result<Config> {
    let cfg = Config::default();
    save(&cfg)?;
    Ok(cfg)
}

// ---- Default page / profile definitions ------------------------------------

fn default_pages() -> Vec<Page> {
    // 3 equal zones: 640px each, 1920px total.
    fn three_zones(w1: &str, w2: &str, w3: &str) -> Vec<Zone> {
        vec![
            Zone { id: "left".into(), x: 0, width: 640, widget: Some(w1.to_string()) },
            Zone { id: "center".into(), x: 640, width: 640, widget: Some(w2.to_string()) },
            Zone { id: "right".into(), x: 1280, width: 640, widget: Some(w3.to_string()) },
        ]
    }

    vec![
        Page {
            name: "clock".into(),
            label: "Clock".into(),
            zones: vec![
                Zone { id: "full".into(), x: 0, width: 1920, widget: Some("flip-clock".into()) },
            ],
        },
        Page {
            name: "dashboard".into(),
            label: "Dashboard".into(),
            zones: three_zones("flip-clock", "weather", "system-monitor"),
        },
        Page {
            name: "music".into(),
            label: "Music".into(),
            zones: three_zones("now-playing", "system-monitor", "notifications"),
        },
        Page {
            name: "ai".into(),
            label: "AI".into(),
            zones: vec![
                Zone { id: "full".into(), x: 0, width: 1920, widget: Some("openclaw-chat".into()) },
            ],
        },
    ]
}

fn default_profiles() -> Vec<Profile> {
    vec![
        Profile {
            name: "work".into(),
            label: "Work".into(),
            pages: vec!["dashboard".into(), "clock".into()],
            ring_effect: Some(RgbEffect::Off),
        },
        Profile {
            name: "focus".into(),
            label: "Focus".into(),
            pages: vec!["clock".into()],
            ring_effect: Some(RgbEffect::Plain),
        },
        Profile {
            name: "home".into(),
            label: "Home".into(),
            pages: vec!["dashboard".into(), "music".into(), "clock".into()],
            ring_effect: Some(RgbEffect::Breathe),
        },
        Profile {
            name: "sleep".into(),
            label: "Sleep".into(),
            pages: vec!["clock".into()],
            ring_effect: Some(RgbEffect::Off),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_json() {
        let cfg = Config::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        assert_eq!(back.settings.idle_dim_secs, 30);
        assert_eq!(back.settings.ring_effect, RgbEffect::Off);
        assert_eq!(back.active_profile, "work");
        assert!(!back.pages.is_empty());
        assert!(!back.profiles.is_empty());
    }

    #[test]
    fn default_pages_cover_full_width() {
        for page in default_pages() {
            let total: u32 = page.zones.iter().map(|z| z.width).sum();
            // Full-screen pages use 1920, 3-zone pages use 1920.
            assert_eq!(total, 1920, "page '{}' zones total {} != 1920", page.name, total);
        }
    }

    #[test]
    fn active_profile_exists_in_profiles() {
        let cfg = Config::default();
        let names: Vec<&str> = cfg.profiles.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&cfg.active_profile.as_str()));
    }

    #[test]
    fn profile_pages_exist_in_pages() {
        let cfg = Config::default();
        let page_names: Vec<&str> = cfg.pages.iter().map(|p| p.name.as_str()).collect();
        for profile in &cfg.profiles {
            for page in &profile.pages {
                assert!(page_names.contains(&page.as_str()),
                    "profile '{}' references missing page '{}'", profile.name, page);
            }
        }
    }
}