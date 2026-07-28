//! Tauri IPC commands callable from the frontend.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::config::{self, Config};
use crate::device::{DeviceState, PowerConfig, PowerState, QuakeDevice};
use crate::notifications::{Notification, NotificationFeed};
use crate::spotify::{self, NowPlaying, PlaybackState};
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

// ---- Spotify ---------------------------------------------------------------

#[tauri::command]
pub fn spotify_auth_status() -> bool {
    spotify::is_authenticated()
}

#[tauri::command]
pub async fn spotify_start_auth() -> Result<String, String> {
    let verifier = spotify::generate_code_verifier();
    let state = spotify::generate_state();
    let url = spotify::auth_url(spotify::CLIENT_ID, &verifier, &state);

    // Spawn the callback server in the background — it will wait for the
    // OAuth redirect, exchange the code for tokens, and save them.
    let verifier_clone = verifier.clone();
    let state_clone = state.clone();
    let client_id = spotify::CLIENT_ID.to_string();
    tauri::async_runtime::spawn(async move {
        match spotify::start_callback_server(state_clone, verifier_clone, client_id).await {
            Ok(_tokens) => log::info!("Spotify auth successful, tokens saved"),
            Err(e) => log::error!("Spotify auth failed: {}", e),
        }
    });

    // Return the auth URL — the frontend opens it in the user's browser.
    Ok(url)
}

#[tauri::command]
pub async fn spotify_exchange_code(code: String, code_verifier: String) -> Result<(), String> {
    let tokens = spotify::exchange_code(spotify::CLIENT_ID, &code, &code_verifier)
        .await?;
    spotify::save_tokens(&tokens).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn spotify_logout() -> Result<(), String> {
    spotify::clear_tokens().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn spotify_now_playing() -> Result<Option<NowPlaying>, String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::get_now_playing(&token).await
}

#[tauri::command]
pub async fn spotify_toggle_play() -> Result<(), String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::toggle_playback(&token).await
}

#[tauri::command]
pub async fn spotify_next() -> Result<(), String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::next_track(&token).await
}

#[tauri::command]
pub async fn spotify_previous() -> Result<(), String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::previous_track(&token).await
}

#[tauri::command]
pub async fn spotify_set_volume(volume: u8) -> Result<(), String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::set_volume(&token, volume).await
}

#[tauri::command]
pub async fn spotify_get_state() -> Result<PlaybackState, String> {
    let token = spotify::ensure_valid_token(spotify::CLIENT_ID).await?;
    spotify::get_playback_state(&token).await
}

// ---- Notifications ---------------------------------------------------------

/// In-memory notification feed shared across the app.
pub static NOTIFICATION_FEED: std::sync::LazyLock<std::sync::Mutex<NotificationFeed>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(NotificationFeed::default()));

#[tauri::command]
pub fn get_notifications() -> Result<Vec<Notification>, String> {
    let feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    Ok(feed.all_sorted().into_iter().cloned().collect())
}

#[tauri::command]
pub fn get_unread_notifications() -> Result<Vec<Notification>, String> {
    let feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    Ok(feed.unread().into_iter().cloned().collect())
}

#[tauri::command]
pub fn get_unread_count() -> Result<usize, String> {
    let feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    Ok(feed.unread_count())
}

#[tauri::command]
pub fn mark_notification_read(id: String) -> Result<(), String> {
    let mut feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    feed.mark_read(&id);
    Ok(())
}

#[tauri::command]
pub fn snooze_notification(id: String) -> Result<(), String> {
    let mut feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    feed.snooze(&id);
    Ok(())
}

#[tauri::command]
pub fn dismiss_notification(id: String) -> Result<(), String> {
    let mut feed = NOTIFICATION_FEED.lock().map_err(|e| e.to_string())?;
    feed.dismiss(&id);
    Ok(())
}

#[tauri::command]
pub async fn fetch_github_notifications() -> Result<usize, String> {
    // TODO: Read token from config. For now, return 0 if no token configured.
    // This will be wired up once notification config is added to settings.
    Ok(0)
}

// ---- System stats ----------------------------------------------------------

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    // sample() blocks ~250ms for a CPU reading; keep it off the async runtime.
    tauri::async_runtime::spawn_blocking(|| stats::sample())
        .await
        .map_err(|e| e.to_string())
}