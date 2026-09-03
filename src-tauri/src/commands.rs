//! Tauri IPC commands callable from the frontend.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::device::{DeviceState, QuakeDevice};
use crate::stats::{self, SystemStats};

/// Response shape for `get_device_info`.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub device_name: u8,
    pub version: String,
}

/// Debug: raw HID device info from hidapi.
#[derive(Debug, Clone, Serialize)]
pub struct HidDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage: u16,
    pub usage_page: u16,
    pub product_string: Option<String>,
    pub manufacturer_string: Option<String>,
    pub path: String,
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

// ---- System stats ----------------------------------------------------------

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    tauri::async_runtime::spawn_blocking(|| stats::sample())
        .await
        .map_err(|e| e.to_string())
}
// ---- Debug -----------------------------------------------------------------

#[tauri::command]
pub fn list_hid_devices() -> Result<Vec<HidDeviceInfo>, String> {
    let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
    let devices: Vec<HidDeviceInfo> = api
        .device_list()
        .map(|d| HidDeviceInfo {
            vendor_id: d.vendor_id(),
            product_id: d.product_id(),
            usage: d.usage(),
            usage_page: d.usage_page(),
            product_string: d.product_string().map(|s| s.to_string()),
            manufacturer_string: d.manufacturer_string().map(|s| s.to_string()),
            path: d.path().to_string_lossy().to_string(),
        })
        .collect();
    Ok(devices)
}