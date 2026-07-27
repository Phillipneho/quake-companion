//! Tauri IPC commands callable from the frontend.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::device::{DeviceState, PowerConfig, PowerState, QuakeDevice};
use crate::stats::{self, SystemStats};
use crate::via::RgbEffect;

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

// ---- System stats ----------------------------------------------------------

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    // sample() blocks ~250ms for a CPU reading; keep it off the async runtime.
    tauri::async_runtime::spawn_blocking(|| stats::sample())
        .await
        .map_err(|e| e.to_string())
}