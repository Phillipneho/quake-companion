//! Home Assistant integration — talks to the HA REST API.
//!
//! Uses a Long-Lived Access Token for auth. The token is stored in the config.
//! Provides: entity state queries, light/switch toggle, climate control,
//! media player control, and sensor data for widgets.

use serde::{Deserialize, Serialize};

// ---- HA API types ----------------------------------------------------------

/// HA entity state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HaEntity {
    pub entity_id: String,
    pub state: String,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(default)]
    pub last_changed: String,
    #[serde(default)]
    pub last_updated: String,
}

impl HaEntity {
    /// Get the friendly_name attribute.
    pub fn friendly_name(&self) -> String {
        self.attributes
            .get("friendly_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.entity_id)
            .to_string()
    }

    /// Get a numeric attribute.
    pub fn attr_num(&self, key: &str) -> Option<f64> {
        self.attributes.get(key).and_then(|v| v.as_f64())
    }

    /// Get a string attribute.
    pub fn attr_str(&self, key: &str) -> Option<String> {
        self.attributes.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }
}

/// A simplified entity for the frontend (less data over IPC).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HaEntitySummary {
    pub entity_id: String,
    pub friendly_name: String,
    pub state: String,
    pub domain: String,
}

impl From<&HaEntity> for HaEntitySummary {
    fn from(e: &HaEntity) -> Self {
        let domain = e.entity_id.split('.').next().unwrap_or("").to_string();
        Self {
            entity_id: e.entity_id.clone(),
            friendly_name: e.friendly_name(),
            state: e.state.clone(),
            domain,
        }
    }
}

// ---- HA client ------------------------------------------------------------

/// Home Assistant client.
pub struct HaClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl HaClient {
    pub fn new(url: &str, token: &str) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            http: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    /// Get all entity states.
    pub async fn get_states(&self) -> Result<Vec<HaEntity>, String> {
        let resp = self
            .http
            .get(self.url("/states"))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("HA get_states failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("HA get_states: {}", resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("HA get_states parse: {e}"))
    }

    /// Get a single entity state.
    pub async fn get_state(&self, entity_id: &str) -> Result<HaEntity, String> {
        let resp = self
            .http
            .get(self.url(&format!("/states/{}", entity_id)))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("HA get_state failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("HA get_state {}: {}", entity_id, resp.status()));
        }

        resp.json()
            .await
            .map_err(|e| format!("HA get_state parse: {e}"))
    }

    /// Call a service (e.g. light.turn_on, media_player.play_media).
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        service_data: serde_json::Value,
    ) -> Result<(), String> {
        let resp = self
            .http
            .post(self.url(&format!("/services/{}/{}", domain, service)))
            .bearer_auth(&self.token)
            .json(&service_data)
            .send()
            .await
            .map_err(|e| format!("HA call_service failed: {e}"))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("HA call_service {domain}.{service}: {body}"));
        }

        Ok(())
    }

    /// Toggle an entity (light, switch, media_player).
    pub async fn toggle(&self, entity_id: &str) -> Result<(), String> {
        let domain = entity_id.split('.').next().unwrap_or("");
        self.call_service(domain, "toggle", serde_json::json!({ "entity_id": entity_id }))
            .await
    }

    /// Turn on an entity.
    pub async fn turn_on(&self, entity_id: &str) -> Result<(), String> {
        let domain = entity_id.split('.').next().unwrap_or("");
        self.call_service(domain, "turn_on", serde_json::json!({ "entity_id": entity_id }))
            .await
    }

    /// Turn off an entity.
    pub async fn turn_off(&self, entity_id: &str) -> Result<(), String> {
        let domain = entity_id.split('.').next().unwrap_or("");
        self.call_service(domain, "turn_off", serde_json::json!({ "entity_id": entity_id }))
            .await
    }

    /// Set light brightness (0-255).
    pub async fn set_brightness(&self, entity_id: &str, brightness: u8) -> Result<(), String> {
        self.call_service(
            "light",
            "turn_on",
            serde_json::json!({ "entity_id": entity_id, "brightness": brightness }),
        )
        .await
    }

    /// Set climate temperature.
    pub async fn set_climate_temp(&self, entity_id: &str, temp: f64) -> Result<(), String> {
        self.call_service(
            "climate",
            "set_temperature",
            serde_json::json!({ "entity_id": entity_id, "temperature": temp }),
        )
        .await
    }

    /// Media player: play/pause.
    pub async fn media_play_pause(&self, entity_id: &str) -> Result<(), String> {
        self.call_service(
            "media_player",
            "media_play_pause",
            serde_json::json!({ "entity_id": entity_id }),
        )
        .await
    }

    /// Media player: next track.
    pub async fn media_next(&self, entity_id: &str) -> Result<(), String> {
        self.call_service(
            "media_player",
            "media_next_track",
            serde_json::json!({ "entity_id": entity_id }),
        )
        .await
    }

    /// Media player: set volume (0-1.0).
    pub async fn set_volume(&self, entity_id: &str, volume: f64) -> Result<(), String> {
        self.call_service(
            "media_player",
            "volume_set",
            serde_json::json!({ "entity_id": entity_id, "volume_level": volume }),
        )
        .await
    }
}

// ---- HA config stored in app config ---------------------------------------

/// HA connection config, stored within the main config's settings.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HaConfig {
    /// HA base URL (e.g. "http://homelab:8123").
    #[serde(default)]
    pub url: Option<String>,
    /// Long-Lived Access Token.
    #[serde(default)]
    pub token: Option<String>,
    /// Entity IDs to show in the QUAKE widget, grouped by domain.
    #[serde(default)]
    pub entities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_summary_extracts_domain() {
        let entity = HaEntity {
            entity_id: "light.living_room".into(),
            state: "on".into(),
            attributes: serde_json::json!({"friendly_name": "Living Room Light"}),
            last_changed: "".into(),
            last_updated: "".into(),
        };
        let summary = HaEntitySummary::from(&entity);
        assert_eq!(summary.domain, "light");
        assert_eq!(summary.friendly_name, "Living Room Light");
        assert_eq!(summary.state, "on");
    }

    #[test]
    fn entity_summary_handles_missing_friendly_name() {
        let entity = HaEntity {
            entity_id: "sensor.temp".into(),
            state: "22.5".into(),
            attributes: serde_json::json!({}),
            last_changed: "".into(),
            last_updated: "".into(),
        };
        let summary = HaEntitySummary::from(&entity);
        assert_eq!(summary.friendly_name, "sensor.temp");
    }

    #[test]
    fn ha_config_roundtrips_json() {
        let cfg = HaConfig {
            url: Some("http://homelab:8123".into()),
            token: Some("test_token".into()),
            entities: vec!["light.living_room".into(), "media_player.kitchen".into()],
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: HaConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.url, Some("http://homelab:8123".into()));
        assert_eq!(back.entities.len(), 2);
    }
}