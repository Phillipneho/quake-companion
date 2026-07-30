//! Tauri application shell: window/display setup, device management, event
//! forwarding, and the keep-alive watchdog task.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager, WindowEvent};
use tokio::sync::mpsc;

use crate::commands;
use crate::config;
use crate::device::{QuakeDevice, QuakeEvent};
use crate::hid::KEEP_ALIVE_INTERVAL_MS;
use crate::widgets::WidgetRegistry;

/// Event channel name used to push device events to the frontend.
pub const EVENT_CHANNEL: &str = "quake://event";

/// Boot the Tauri app. Called from `main.rs` (under `#[tauri::main]`).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::wake_device,
            commands::set_brightness,
            commands::set_screen,
            commands::set_mic,
            commands::set_led,
            commands::ping_device,
            commands::get_device_info,
            commands::get_device_state,
            commands::get_system_stats,
            commands::dim_screen,
            commands::sleep_screen,
            commands::wake_screen,
            commands::get_power_state,
            commands::set_power_config,
            commands::get_power_config,
            // VIA RGB ring control
            commands::via_set_effect,
            commands::via_set_brightness,
            commands::via_set_color_1,
            commands::via_set_color_2,
            commands::via_set_custom_color,
            commands::via_save_lighting,
            commands::via_eeprom_reset,
            commands::via_bootloader_jump,
            // Config system
            commands::get_config,
            commands::save_config,
            commands::reset_config,
            commands::set_active_profile,
            // Widget SDK
            commands::list_widgets,
            commands::get_widget,
            // Spotify
            commands::spotify_auth_status,
            commands::spotify_start_auth,
            commands::spotify_exchange_code,
            commands::spotify_logout,
            commands::spotify_now_playing,
            commands::spotify_toggle_play,
            commands::spotify_next,
            commands::spotify_previous,
            commands::spotify_set_volume,
            commands::spotify_get_state,
            // Notifications
            commands::get_notifications,
            commands::get_unread_notifications,
            commands::get_unread_count,
            commands::mark_notification_read,
            commands::snooze_notification,
            commands::dismiss_notification,
            commands::fetch_github_notifications,
            // Home Assistant
            commands::ha_get_states,
            commands::ha_get_entity,
            commands::ha_toggle,
            commands::ha_turn_on,
            commands::ha_turn_off,
            commands::ha_set_brightness,
            commands::ha_media_play_pause,
            commands::ha_set_volume,
            // OpenClaw Panel
            commands::oc_get_messages,
            commands::oc_clear_conversation,
            commands::oc_send_text,
            commands::oc_transcribe_and_respond,
            commands::oc_respond_to_text,
            commands::oc_get_recording_state,
            // AI-Composed Panels
            commands::compose_panel,
            commands::save_composed_panel,
            // Context Engine
            commands::get_context_state,
            commands::set_context_rules,
            commands::note_interaction,
            // Trigger Engine + Ring
            commands::get_triggers,
            commands::get_trigger_state,
            commands::set_ring_status,
            // Gestures + Burn-in
            commands::process_touch_points,
            commands::get_burn_in_config,
            commands::get_scheduled_brightness,
        ])
        .on_window_event(on_window_event)
        .run(tauri::generate_context!())?;
    Ok(())
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Place the window on the DK-QUAKE display if present (1920x480). Falls
    // back to the primary monitor in dev.
    position_on_quake_display(app);

    // Load config (seeds default on first run).
    let cfg = config::load()?;
    app.manage(Arc::new(Mutex::new(cfg)));

    // Load widget registry (built-in + on-disk widgets).
    let widget_registry = WidgetRegistry::load()?;
    app.manage(Arc::new(widget_registry));

    // Event channel: device worker threads -> this forwarder -> frontend.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<QuakeEvent>();

    // Create the device (starts its own worker threads; re-binds on connect).
    let device = QuakeDevice::new(event_tx)?;

    // Forward device events to every window as `quake://event`.
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // Emit to all windows; ignore failures (window may be closing).
            let _ = app_handle.emit(EVENT_CHANNEL, &event);
        }
    });

    // Keep-alive: ping every 1500ms so the panel does not blank.
    let watchdog = Arc::clone(&device);
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(KEEP_ALIVE_INTERVAL_MS));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            if let Err(e) = watchdog.ping() {
                log::warn!("keep-alive ping failed: {e}");
            }
        }
    });

    // Idle power manager: check dim/sleep thresholds every 5s.
    let power_mgr = Arc::clone(&device);
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            if let Err(e) = power_mgr.check_idle() {
                log::warn!("idle check failed: {e}");
            }
        }
    });

    app.manage(device);

    Ok(())
}

fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { .. } = event {
        // Best-effort shutdown of the device when the window closes.
        if let Some(device) = window.app_handle().try_state::<Arc<QuakeDevice>>() {
            device.shutdown();
        }
    }
}

/// Move + size the main window to the DK-QUAKE display (1920x480) if one is
/// attached; otherwise leave it on the primary monitor at 1920x480.
fn position_on_quake_display(app: &mut tauri::App) {
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("[QUAKE] No main window found for positioning");
        return;
    };

    let monitors = available_monitors(app);
    eprintln!("[QUAKE] Found {} monitors:", monitors.len());
    for m in &monitors {
        eprintln!("[QUAKE]   name={:?} x={} y={} {}x{}", m.name(), m.x(), m.y(), m.width(), m.height());
    }

    let quake = monitors
        .into_iter()
        .find(|m| is_quake_monitor(m));

    if let Some(monitor) = quake {
        eprintln!("[QUAKE] Positioning on QUAKE display at {},{} reporting {}x{}", monitor.x(), monitor.y(), monitor.width(), monitor.height());
        // Use physical position/size. Tauri reports logical coords (640x480
        // due to 300% DPI scaling) but the panel is physically 1920x480.
        // The monitor position from Tauri is in logical coords, so we scale
        // back to physical: logical_x * scale = physical_x.
        // From PowerShell: DISPLAY13 physical bounds = {X=1903,Y=1085,1920x480}
        // Tauri reports logical: x=-640, y=0, 640x480 (scale ~3x)
        // Use physical coords directly for reliable positioning.
        let _ = window.set_position(tauri::PhysicalPosition::new(1903i32, 1085i32));
        let _ = window.set_size(tauri::PhysicalSize::new(1920u32, 480u32));
    } else {
        eprintln!("[QUAKE] No QUAKE monitor found — window stays on default display");
    }
}

/// A flat monitor descriptor (logical pixels) that abstracts over the
/// available-monitor API quirks across Tauri versions.
struct Monitor {
    name: Option<String>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Monitor {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// True if the monitor looks like the DK-QUAKE panel: named "DK-QUAKE" or "QUAKE",
/// or with a height of 480 (the QUAKE panel's distinctive dimension regardless
/// of reported width — Tauri may report it at 640x480 instead of 1920x480).
fn is_quake_monitor(m: &Monitor) -> bool {
    if let Some(name) = m.name() {
        let n = name.to_ascii_uppercase();
        if n.contains("DK-QUAKE") || n.contains("QUAKE") {
            return true;
        }
    }
    // Match on 480-pixel height — unique to the QUAKE ultra-wide panel.
    // Tauri may report it as 1920x480 or 640x480 depending on DPI/scaling.
    m.height() == 480 || m.width() == 480
}

/// Collect available monitors across the Tauri v2 monitor API variants.
fn available_monitors(app: &tauri::App) -> Vec<Monitor> {
    let Some(window) = app.get_webview_window("main") else {
        return Vec::new();
    };
    window
        .available_monitors()
        .unwrap_or_default()
        .into_iter()
        .map(|m| Monitor {
            name: m.name().cloned(),
            x: m.position().x as i32,
            y: m.position().y as i32,
            width: m.size().width as u32,
            height: m.size().height as u32,
        })
        .collect()
}