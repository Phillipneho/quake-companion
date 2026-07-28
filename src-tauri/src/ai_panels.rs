//! AI-composed panels — LLM generates zone layouts + widget configs from natural language.
//!
//! The user describes what they want (e.g. "I want a dashboard with the clock,
//! weather, and system stats"), and the LLM returns a JSON page definition that
//! fits into the existing config system. The page renders immediately.

use serde::{Deserialize, Serialize};

use crate::config::{Config, Page, Zone};

// ---- AI panel request/response types ---------------------------------------

/// Request to compose a panel from natural language.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposePanelRequest {
    /// Natural language description of what the user wants.
    pub prompt: String,
    /// Available widget ids (from the registry).
    pub available_widgets: Vec<String>,
    /// Display width in pixels (default 1920).
    pub display_width: u32,
}

/// The LLM's response — a page definition ready to insert into config.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposedPanel {
    pub name: String,
    pub label: String,
    pub zones: Vec<Zone>,
    /// The raw LLM response for debugging.
    pub raw_response: String,
}

// ---- Panel composition ----------------------------------------------------

/// Compose a panel from a natural language prompt.
/// Calls the LLM with the available widgets and asks it to produce a JSON page definition.
pub async fn compose_panel(req: ComposePanelRequest) -> Result<ComposedPanel, String> {
    // Build the system prompt with available widgets
    let widget_list = req.available_widgets.join(", ");
    let _system_prompt = format!(
        "You are a UI layout designer for the QUAKE Companion — a 1920×480 ultra-wide display panel. \
         The user will describe what they want to see on the panel. You must respond with ONLY a JSON object \
         (no markdown, no explanation) with this exact structure:\n\
         {{\n\
           \"name\": \"short-kebab-name\",\n\
           \"label\": \"Human Readable Label\",\n\
           \"zones\": [\n\
             {{\"id\": \"left\", \"x\": 0, \"width\": 640, \"widget\": \"widget-id\"}},\n\
             {{\"id\": \"center\", \"x\": 640, \"width\": 640, \"widget\": \"widget-id\"}},\n\
             {{\"id\": \"right\", \"x\": 1280, \"width\": 640, \"widget\": \"widget-id\"}}\n\
           ]\n\
         }}\n\n\
         Rules:\n\
         - Total zone widths must equal {display_width} pixels\n\
         - Use 1-3 zones. Full-screen pages use one zone spanning the full width.\n\
         - Widget ids must be one of: {widgets}\n\
         - Zone x positions must be cumulative (first zone starts at 0)\n\
         - Keep the name short (kebab-case), label human-readable\n\
         - Respond with ONLY the JSON, nothing else",
        display_width = req.display_width,
        widgets = widget_list,
    );

    // Call the LLM (placeholder — will be replaced with OpenClaw gateway call)
    // For now, we do a simple keyword-matching heuristic so the feature works end-to-end.
    let page = compose_heuristic(&req.prompt, &req.available_widgets, req.display_width);

    Ok(ComposedPanel {
        name: page.name,
        label: page.label,
        zones: page.zones,
        raw_response: format!("Heuristic composition for: {}", req.prompt),
    })
}

/// Heuristic panel composition — keyword matching fallback.
/// This will be replaced by the LLM call once the streaming integration is wired up.
fn compose_heuristic(prompt: &str, available_widgets: &[String], display_width: u32) -> Page {
    let p = prompt.to_lowercase();

    // Detect which widgets the user wants
    let mut wanted: Vec<&str> = Vec::new();

    let keywords: &[(&str, &str)] = &[
        ("clock", "flip-clock"),
        ("time", "flip-clock"),
        ("weather", "weather"),
        ("temperature", "weather"),
        ("system", "system-monitor"),
        ("cpu", "system-monitor"),
        ("stats", "system-monitor"),
        ("monitor", "system-monitor"),
        ("music", "now-playing"),
        ("spotify", "now-playing"),
        ("playing", "now-playing"),
        ("notification", "notifications"),
        ("notif", "notifications"),
        ("message", "notifications"),
        ("home", "home-assistant"),
        ("light", "home-assistant"),
        ("smart", "home-assistant"),
        ("assistant", "home-assistant"),
        ("ai", "openclaw-chat"),
        ("chat", "openclaw-chat"),
        ("openclaw", "openclaw-chat"),
        ("leo", "openclaw-chat"),
        ("calendar", "calendar"),
        ("event", "calendar"),
        ("meeting", "calendar"),
        ("pomodoro", "pomodoro"),
        ("focus", "pomodoro"),
        ("timer", "pomodoro"),
        ("stoic", "daily-stoic"),
        ("quote", "daily-stoic"),
        ("philosophy", "daily-stoic"),
        ("web", "web-dashboard"),
        ("dashboard", "web-dashboard"),
        ("grafana", "web-dashboard"),
    ];

    for (kw, widget_id) in keywords {
        if p.contains(kw) && available_widgets.contains(&widget_id.to_string()) && !wanted.contains(widget_id) {
            wanted.push(widget_id);
        }
    }

    // If nothing detected, default to clock
    if wanted.is_empty() {
        wanted.push("flip-clock");
    }

    // Limit to 3 widgets (3 zones max)
    wanted.truncate(3);

    // Build zones
    let count = wanted.len();
    let zone_width = display_width / count as u32;

    let zones: Vec<Zone> = wanted
        .iter()
        .enumerate()
        .map(|(i, widget_id)| {
            let id = match i {
                0 => "left",
                1 => "center",
                2 => "right",
                _ => "zone",
            };
            Zone {
                id: id.to_string(),
                x: (i as u32) * zone_width,
                width: zone_width,
                widget: Some(widget_id.to_string()),
            }
        })
        .collect();

    // Generate name + label
    let label = if count == 1 {
        match wanted[0] {
            "flip-clock" => "Clock".to_string(),
            "openclaw-chat" => "AI Chat".to_string(),
            "web-dashboard" => "Web Dashboard".to_string(),
            _ => wanted[0].replace('-', " ").to_title_case(),
        }
    } else {
        format!("AI Panel · {} widgets", count)
    };

    let name = format!("ai-{}", chrono::Utc::now().timestamp() % 100000);

    Page {
        name,
        label,
        zones,
    }
}

/// Save a composed panel into the config.
pub fn save_composed_panel(config: &mut Config, panel: Page) -> Result<(), String> {
    // Add or replace the page
    if let Some(existing) = config.pages.iter_mut().find(|p| p.name == panel.name) {
        *existing = panel.clone();
    } else {
        config.pages.push(panel.clone());
    }

    // Add the page to the active profile's page list if not already there
    if let Some(profile) = config.profiles.iter_mut().find(|p| p.name == config.active_profile) {
        if !profile.pages.contains(&panel.name) {
            profile.pages.push(panel.name.clone());
        }
    }

    Ok(())
}

// ---- String helper ---------------------------------------------------------

trait ToTitleCase {
    fn to_title_case(self) -> String;
}

impl ToTitleCase for String {
    fn to_title_case(self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_widgets() -> Vec<String> {
        vec![
            "flip-clock".into(),
            "weather".into(),
            "system-monitor".into(),
            "now-playing".into(),
            "notifications".into(),
            "openclaw-chat".into(),
            "home-assistant".into(),
            "calendar".into(),
            "pomodoro".into(),
            "daily-stoic".into(),
            "web-dashboard".into(),
        ]
    }

    #[test]
    fn heuristic_detects_single_widget() {
        let page = compose_heuristic("show me the clock", &make_widgets(), 1920);
        assert_eq!(page.zones.len(), 1);
        assert_eq!(page.zones[0].widget, Some("flip-clock".into()));
        assert_eq!(page.zones[0].width, 1920);
    }

    #[test]
    fn heuristic_detects_three_widgets() {
        let page = compose_heuristic("clock weather and system stats", &make_widgets(), 1920);
        assert_eq!(page.zones.len(), 3);
        assert_eq!(page.zones[0].widget, Some("flip-clock".into()));
        assert_eq!(page.zones[1].widget, Some("weather".into()));
        assert_eq!(page.zones[2].widget, Some("system-monitor".into()));
        // Total width = 1920
        let total: u32 = page.zones.iter().map(|z| z.width).sum();
        assert_eq!(total, 1920);
    }

    #[test]
    fn heuristic_defaults_to_clock() {
        let page = compose_heuristic("something nice", &make_widgets(), 1920);
        assert_eq!(page.zones.len(), 1);
        assert_eq!(page.zones[0].widget, Some("flip-clock".into()));
    }

    #[test]
    fn heuristic_respects_display_width() {
        let page = compose_heuristic("clock weather", &make_widgets(), 1280);
        assert_eq!(page.zones.len(), 2);
        let total: u32 = page.zones.iter().map(|z| z.width).sum();
        assert_eq!(total, 1280);
    }

    #[test]
    fn save_composed_panel_adds_page_and_updates_profile() {
        let mut config = Config::default();
        let initial_pages = config.pages.len();

        let panel = Page {
            name: "ai-test".into(),
            label: "AI Test".into(),
            zones: vec![Zone {
                id: "full".into(),
                x: 0,
                width: 1920,
                widget: Some("flip-clock".into()),
            }],
        };

        save_composed_panel(&mut config, panel).unwrap();

        assert_eq!(config.pages.len(), initial_pages + 1);
        let active_profile = config.active_profile.clone();
        let profile = config.profiles.iter().find(|p| p.name == active_profile).unwrap();
        assert!(profile.pages.contains(&"ai-test".to_string()));
    }

    #[test]
    fn save_composed_panel_replaces_existing() {
        let mut config = Config::default();

        let panel1 = Page {
            name: "ai-test".into(),
            label: "V1".into(),
            zones: vec![Zone { id: "full".into(), x: 0, width: 1920, widget: Some("flip-clock".into()) }],
        };
        save_composed_panel(&mut config, panel1).unwrap();

        let panel2 = Page {
            name: "ai-test".into(),
            label: "V2".into(),
            zones: vec![Zone { id: "full".into(), x: 0, width: 1920, widget: Some("weather".into()) }],
        };
        save_composed_panel(&mut config, panel2).unwrap();

        // Should not have added a duplicate
        let count = config.pages.iter().filter(|p| p.name == "ai-test").count();
        assert_eq!(count, 1);
        // Should have the updated label
        let page = config.pages.iter().find(|p| p.name == "ai-test").unwrap();
        assert_eq!(page.label, "V2");
    }
}