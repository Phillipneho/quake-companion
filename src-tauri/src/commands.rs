//! Tauri IPC commands callable from the frontend.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::config::{self, Config};
use crate::device::{DeviceState, PowerConfig, PowerState, QuakeDevice};
use crate::stats::{self, SystemStats};
use crate::via::RgbEffect;
use crate::widgets::{WidgetManifest, WidgetRegistry};

/// Response shape for `get_device_info`.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub device_name: u8,
    pub version: String,
}

// ---- Device control --------------------------------------------------------

#[tauri::command]
pub fn wake_device(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.wake(255).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_brightness(device: State<'_, Arc<QuakeDevice>>, value: u8) -> Result<(), String> {
    device.set_brightness(value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_screen(device: State<'_, Arc<QuakeDevice>>, on: bool) -> Result<(), String> {
    device.set_screen(on).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_mic(device: State<'_, Arc<QuakeDevice>>, on: bool) -> Result<(), String> {
    device.set_mic(on).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_led(device: State<'_, Arc<QuakeDevice>>, mode: u8) -> Result<(), String> {
    device.set_led(mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ping_device(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.ping().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_device_info(
    device: State<'_, Arc<QuakeDevice>>,
) -> Result<DeviceInfo, String> {
    // Clone the Arc before awaiting so we don't hold a borrow of Tauri state.
    let dev = device.inner().clone();
    let (device_name, version) = dev.get_info().await.map_err(|e| e.to_string())?;
    Ok(DeviceInfo {
        device_name,
        version,
    })
}

/// Snapshot of the cached hardware state (no HID round-trip; always succeeds).
#[tauri::command]
pub fn get_device_state(device: State<'_, Arc<QuakeDevice>>) -> DeviceState {
    device.state()
}

// ---- Power management ------------------------------------------------------

#[tauri::command]
pub fn dim_screen(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.dim().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sleep_screen(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.sleep_screen().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn wake_screen(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.wake_from_idle().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_power_state(device: State<'_, Arc<QuakeDevice>>) -> PowerState {
    device.power_state()
}

#[tauri::command]
pub fn set_power_config(
    device: State<'_, Arc<QuakeDevice>>,
    config: PowerConfig,
) -> Result<(), String> {
    device.set_power_config(config);
    Ok(())
}

#[tauri::command]
pub fn get_power_config(device: State<'_, Arc<QuakeDevice>>) -> PowerConfig {
    device.power_config()
}

// ---- VIA RGB ring control -------------------------------------------------

#[tauri::command]
pub fn via_set_effect(device: State<'_, Arc<QuakeDevice>>, effect: RgbEffect) -> Result<(), String> {
    device.via_set_effect(effect).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_set_brightness(device: State<'_, Arc<QuakeDevice>>, brightness: u8) -> Result<(), String> {
    device.via_set_brightness(brightness).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_set_color_1(device: State<'_, Arc<QuakeDevice>>, hue: u8, sat: u8) -> Result<(), String> {
    device.via_set_color_1(hue, sat).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_set_color_2(device: State<'_, Arc<QuakeDevice>>, hue: u8, sat: u8) -> Result<(), String> {
    device.via_set_color_2(hue, sat).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_set_custom_color(device: State<'_, Arc<QuakeDevice>>, index: u8, hue: u8, sat: u8) -> Result<(), String> {
    device.via_set_custom_color(index, hue, sat).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_save_lighting(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.via_save_lighting().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_eeprom_reset(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.via_eeprom_reset().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn via_bootloader_jump(device: State<'_, Arc<QuakeDevice>>) -> Result<(), String> {
    device.via_bootloader_jump().map_err(|e| e.to_string())
}

// ---- Config system --------------------------------------------------------

#[tauri::command]
pub fn get_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<Config, String> {
    let cfg = config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn save_config(
    config: State<'_, Arc<Mutex<Config>>>,
    new_config: Config,
) -> Result<(), String> {
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        *cfg = new_config;
    }
    // Persist to disk
    let cfg = config.lock().map_err(|e| e.to_string())?;
    config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<Config, String> {
    let new_cfg = config::reset().map_err(|e| e.to_string())?;
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        *cfg = new_cfg.clone();
    }
    Ok(new_cfg)
}

#[tauri::command]
pub fn set_active_profile(
    config: State<'_, Arc<Mutex<Config>>>,
    profile: String,
) -> Result<(), String> {
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        // Validate profile exists
        if !cfg.profiles.iter().any(|p| p.name == profile) {
            return Err(format!("profile '{}' not found", profile));
        }
        cfg.active_profile = profile;
    }
    let cfg = config.lock().map_err(|e| e.to_string())?;
    config::save(&cfg).map_err(|e| e.to_string())
}

// ---- Widget SDK -----------------------------------------------------------

#[tauri::command]
pub fn list_widgets(registry: State<'_, Arc<WidgetRegistry>>) -> Vec<WidgetManifest> {
    registry.list().into_iter().cloned().collect()
}

#[tauri::command]
pub fn get_widget(registry: State<'_, Arc<WidgetRegistry>>, id: String) -> Result<WidgetManifest, String> {
    registry.get(&id).cloned().ok_or_else(|| format!("widget '{}' not found", id))
}

// ---- System stats ----------------------------------------------------------

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    // sample() blocks ~250ms for a CPU reading; keep it off the async runtime.
    tauri::async_runtime::spawn_blocking(|| stats::sample())
        .await
        .map_err(|e| e.to_string())
}