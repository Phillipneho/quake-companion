//! QuakeDevice — high-level handle to a Decokee QUAKE display over raw HID.
//!
//! Wraps the hidapi crate with the 0xA3 control protocol (see `hid.rs`):
//! screen, brightness, mic, LED, info, the watchdog keep-alive, and touch input
//! re-emitted from a separate HID device. This module is independent of Tauri;
//! it emits events through a tokio unbounded channel and can be driven by any
//! host (CLI, tests, or the Tauri shell in `main.rs`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::hid::*;

/// Events pushed from the device worker threads to the host.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum QuakeEvent {
    /// Control + keep-alive are alive (re)connected.
    Connected,
    /// Control device dropped / write failed and will be re-bound.
    Disconnected,
    /// Jog wheel rotation. `direction` is +1 (CW) or -1 (CCW).
    Rotate { direction: i8 },
    /// Jog wheel press. `value` is the raw sub-data byte.
    Press { value: u8 },
    /// Jog wheel held for `duration_ms` then released.
    KnobHold { duration_ms: u64 },
    /// Multi-touch report from the touch HID device.
    Touch { points: Vec<TouchPoint> },
    /// INFO response: device-name byte + firmware `major.minor.patch`.
    Info { device_name: u8, version: String },
    /// BRIGHTNESS response.
    Brightness { value: u8 },
    /// MIC response.
    Mic { enabled: bool },
    /// Keep-alive pong.
    Pong,
    /// Generic change-result (cmdID 0): 0x90 signals success.
    Result { success: bool },
    /// Power state changed (dim/sleep/awake).
    PowerStateChanged { state: PowerState },
}

/// Screen power state for idle management.
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PowerState {
    /// Full brightness, fully interactive.
    #[default]
    Awake,
    /// Dimmed — reduced brightness after idle-dim timeout.
    Dim,
    /// Screen off — after idle-sleep timeout.
    Sleep,
}

/// Configurable idle power management thresholds.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Seconds of inactivity before dimming.
    pub idle_dim_secs: u64,
    /// Seconds of inactivity before sleeping (from full awake).
    pub idle_sleep_secs: u64,
    /// Brightness level when dimmed.
    pub dim_brightness: u8,
    /// Brightness restored on wake.
    pub wake_brightness: u8,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            idle_dim_secs: IDLE_DIM_SECS_DEFAULT,
            idle_sleep_secs: IDLE_SLEEP_SECS_DEFAULT,
            dim_brightness: DIM_BRIGHTNESS_DEFAULT,
            wake_brightness: WAKE_BRIGHTNESS_DEFAULT,
        }
    }
}

/// Last-known cached hardware state, updated from incoming 0x55 reports.
#[derive(Clone, Debug, Default, Serialize)]
pub struct DeviceState {
    pub connected: bool,
    pub version: Option<String>,
    pub device_name: Option<u8>,
    pub brightness: Option<u8>,
    pub mic: Option<bool>,
    /// LED/buzzer effect has no read-back; track the last set value.
    pub led: bool,
    /// Current power state (awake/dim/sleep).
    pub power_state: PowerState,
}

/// A pending query waiting for a matching STATE (0x55) response.
struct QueryWaiter {
    cmd_id: u8,
    tx: oneshot::Sender<Report>,
}

pub struct QuakeDevice {
    api: Mutex<HidApi>,
    control: Mutex<Option<HidDevice>>,
    touch: Mutex<Option<HidDevice>>,
    state: Mutex<DeviceState>,
    waiters: Mutex<Vec<QueryWaiter>>,
    event_tx: mpsc::UnboundedSender<QuakeEvent>,
    running: Arc<AtomicBool>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    /// Timestamp of the last user interaction (touch or knob).
    last_interaction: Mutex<Instant>,
    /// Idle power management configuration.
    power_config: Mutex<PowerConfig>,
    /// Current power state (awake/dim/sleep).
    power_state: Mutex<PowerState>,
    /// Knob press start timestamp for hold detection.
    knob_press_start: Mutex<Option<Instant>>,
}

impl QuakeDevice {
    /// Create a new handle. Attempts to open the control device immediately;
    /// if none is present the worker loop keeps retrying in the background.
    /// The caller owns the `event_rx` half of the channel.
    pub fn new(event_tx: mpsc::UnboundedSender<QuakeEvent>) -> std::io::Result<Arc<Self>> {
        let api = HidApi::new().map_err(|e| std::io::Error::other(e.to_string()))?;
        let running = Arc::new(AtomicBool::new(true));
        let dev = Arc::new(Self {
            api: Mutex::new(api),
            control: Mutex::new(None),
            touch: Mutex::new(None),
            state: Mutex::new(DeviceState::default()),
            waiters: Mutex::new(Vec::new()),
            event_tx,
            running,
            threads: Mutex::new(Vec::new()),
            last_interaction: Mutex::new(Instant::now()),
            power_config: Mutex::new(PowerConfig::default()),
            power_state: Mutex::new(PowerState::Awake),
            knob_press_start: Mutex::new(None),
        });

        // Spawn the control read + keep-alive loop. It opens the control
        // device on first iteration (or re-binds if missing/disconnected).
        dev.spawn_control_loop();
        // The touch device is independent of the control device; spin its loop
        // once — it self-binds / re-binds as the touch HID appears.
        dev.spawn_touch_loop();
        Ok(dev)
    }

    /// Stop all worker loops. The device handles are closed when dropped.
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.event_tx.send(QuakeEvent::Disconnected);
    }

    // ---- Public control API ------------------------------------------------

    /// Send a control frame: payload bytes + flag (1 = set, 2 = query).
    pub fn send(&self, payload: &[u8], flag: u8) -> std::io::Result<()> {
        let report = to_report(payload, flag);
        let mut guard = self.control.lock().unwrap();
        match guard.as_mut() {
            Some(d) => d
                .write(&report)
                .map(|_| ())
                .map_err(|e| std::io::Error::other(e.to_string())),
            None => Err(std::io::Error::other("QUAKE control device not open")),
        }
    }

    /// Send a query (flag 2) and await the matching STATE (0x55) response.
    pub async fn query(&self, cmd_id: u8) -> std::io::Result<Report> {
        let (tx, rx) = oneshot::channel();
        {
            let mut waiters = self.waiters.lock().unwrap();
            waiters.push(QueryWaiter { cmd_id, tx });
        }
        self.send(&[cmd_id], FLAG_QUERY)?;
        match tokio::time::timeout(Duration::from_millis(800), rx).await {
            Ok(Ok(report)) => Ok(report),
            Ok(Err(_)) => Err(std::io::Error::other("QUAKE query channel closed")),
            Err(_) => {
                // Drop the waiter so it doesn't leak.
                let mut waiters = self.waiters.lock().unwrap();
                waiters.retain(|w| w.cmd_id != cmd_id || !w.tx.is_closed());
                Err(std::io::Error::other(format!(
                    "QUAKE query 0x{:x} timed out",
                    cmd_id
                )))
            }
        }
    }

    /// Wake sequence: screen on, full brightness, start keep-alive loop.
    pub fn wake(&self, brightness: u8) -> std::io::Result<()> {
        self.set_screen(true)?;
        self.set_brightness(brightness)?;
        Ok(())
    }

    pub fn set_screen(&self, on: bool) -> std::io::Result<()> {
        self.send(&[CTL_SCREEN, on as u8], FLAG_SET)
    }

    pub fn set_brightness(&self, value: u8) -> std::io::Result<()> {
        self.state.lock().unwrap().brightness = Some(value);
        self.send(&[CTL_BRIGHTNESS, value], FLAG_SET)
    }

    pub async fn get_brightness(&self) -> std::io::Result<u8> {
        let r = self.query(CTL_BRIGHTNESS).await?;
        Ok(*r.sub_data.first().unwrap_or(&0))
    }

    pub fn set_mic(&self, on: bool) -> std::io::Result<()> {
        self.state.lock().unwrap().mic = Some(on);
        self.send(&[CTL_MIC, on as u8], FLAG_SET)
    }

    pub fn set_led(&self, mode: u8) -> std::io::Result<()> {
        self.state.lock().unwrap().led = mode != 0;
        self.send(&[CTL_LED, mode], FLAG_SET)
    }

    pub async fn get_info(&self) -> std::io::Result<(u8, String)> {
        let r = self.query(CTL_INFO).await?;
        let name = *r.sub_data.first().unwrap_or(&0);
        let version = match r.sub_data.get(1..4) {
            Some(v) => format!("{}.{}.{}", v[0], v[1], v[2]),
            None => "0.0.0".to_string(),
        };
        Ok((name, version))
    }

    pub fn ping(&self) -> std::io::Result<()> {
        self.send(&[CTL_KEEP_ALIVE], FLAG_QUERY)
    }

    /// Snapshot of the cached hardware state.
    pub fn state(&self) -> DeviceState {
        self.state.lock().unwrap().clone()
    }

    pub fn is_connected(&self) -> bool {
        self.control.lock().unwrap().is_some()
    }

    // ---- Idle + power management -------------------------------------------

    /// Record a user interaction (touch or knob). Resets the idle timer and
    /// wakes the screen if it was dimmed or sleeping.
    pub fn note_interaction(&self) {
        *self.last_interaction.lock().unwrap() = Instant::now();
        let state = *self.power_state.lock().unwrap();
        if state != PowerState::Awake {
            let _ = self.wake_from_idle();
        }
    }

    /// Returns how long since the last user interaction.
    pub fn idle_duration(&self) -> Duration {
        self.last_interaction.lock().unwrap().elapsed()
    }

    /// Returns the current power state.
    pub fn power_state(&self) -> PowerState {
        *self.power_state.lock().unwrap()
    }

    /// Update power management configuration at runtime.
    pub fn set_power_config(&self, config: PowerConfig) {
        *self.power_config.lock().unwrap() = config;
    }

    /// Get a copy of the current power config.
    pub fn power_config(&self) -> PowerConfig {
        self.power_config.lock().unwrap().clone()
    }

    /// Dim the screen to the configured dim brightness.
    pub fn dim(&self) -> std::io::Result<()> {
        let cfg = self.power_config.lock().unwrap().clone();
        self.set_brightness(cfg.dim_brightness)?;
        self.set_power_state(PowerState::Dim);
        Ok(())
    }

    /// Turn the screen off entirely.
    pub fn sleep_screen(&self) -> std::io::Result<()> {
        self.set_screen(false)?;
        self.set_power_state(PowerState::Sleep);
        Ok(())
    }

    /// Wake from dim/sleep: screen on + full brightness.
    pub fn wake_from_idle(&self) -> std::io::Result<()> {
        let cfg = self.power_config.lock().unwrap().clone();
        self.set_screen(true)?;
        self.set_brightness(cfg.wake_brightness)?;
        self.set_power_state(PowerState::Awake);
        Ok(())
    }

    /// Check idle state and transition power if thresholds are exceeded.
    /// Called periodically by the power manager task in app.rs.
    pub fn check_idle(&self) -> std::io::Result<()> {
        let cfg = self.power_config.lock().unwrap().clone();
        let current = *self.power_state.lock().unwrap();
        let idle = self.idle_duration();

        // Only transition if currently awake or dim.
        match current {
            PowerState::Awake => {
                if idle >= Duration::from_secs(cfg.idle_sleep_secs) {
                    self.sleep_screen()?;
                } else if idle >= Duration::from_secs(cfg.idle_dim_secs) {
                    self.dim()?;
                }
            }
            PowerState::Dim => {
                if idle >= Duration::from_secs(cfg.idle_sleep_secs) {
                    self.sleep_screen()?;
                }
            }
            PowerState::Sleep => {
                // Already sleeping; wake is handled by note_interaction().
            }
        }
        Ok(())
    }

    fn set_power_state(&self, state: PowerState) {
        *self.power_state.lock().unwrap() = state;
        self.state.lock().unwrap().power_state = state;
        let _ = self.event_tx.send(QuakeEvent::PowerStateChanged { state });
    }

    // ---- Worker threads ----------------------------------------------------

    fn spawn_control_loop(self: &Arc<Self>) {
        let dev = Arc::clone(self);
        let handle = thread::Builder::new()
            .name("quake-control".into())
            .spawn(move || dev.control_loop())
            .expect("spawn control loop");
        self.threads.lock().unwrap().push(handle);
    }

    fn spawn_touch_loop(self: &Arc<Self>) {
        let dev = Arc::clone(self);
        let handle = thread::Builder::new()
            .name("quake-touch".into())
            .spawn(move || dev.touch_loop())
            .expect("spawn touch loop");
        self.threads.lock().unwrap().push(handle);
    }

    fn control_loop(&self) {
        let mut buf = [0u8; 64];
        loop {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            // If we don't have a control device, (re)bind.
            let needs_rebind = self.control.lock().unwrap().is_none();
            if needs_rebind {
                if !self.rebind_control() {
                    // No device yet; back off before retrying.
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }
            }

            // Read with a short timeout so writes from other threads get a
            // turn and we can observe `running` changes promptly.
            let outcome = {
                let mut guard = self.control.lock().unwrap();
                match guard.as_mut() {
                    Some(d) => match d.read_timeout(&mut buf, 100) {
                        Ok(0) => ReadOutcome::Empty,
                        Ok(n) => ReadOutcome::Data(buf[..n].to_vec()),
                        Err(_) => ReadOutcome::Error,
                    },
                    None => ReadOutcome::Empty,
                }
            };

            match outcome {
                ReadOutcome::Data(bytes) => self.handle_control_report(&bytes),
                ReadOutcome::Empty => {}
                ReadOutcome::Error => {
                    self.handle_control_error();
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    fn handle_control_report(&self, data: &[u8]) {
        let report = match parse_report(data) {
            Some(r) => r,
            None => return,
        };

        if report.op_code == RESP_KNOB {
            match report.cmd_id {
                1 => {
                    let dir: i8 = if report.sub_data.first().copied() == Some(1) {
                        1
                    } else {
                        -1
                    };
                    self.note_interaction();
                    let _ = self.event_tx.send(QuakeEvent::Rotate { direction: dir });
                }
                2 => {
                    let v = report.sub_data.first().copied().unwrap_or(0);
                    self.note_interaction();

                    // Knob hold detection: firmware sends value=5 at hold start
                    // and value=0xFF at hold end. Track the timestamp to compute
                    // duration. For a simple press (other values), emit Press.
                    if v == KNOB_HOLD_START {
                        *self.knob_press_start.lock().unwrap() = Some(Instant::now());
                    } else if v == KNOB_HOLD_END {
                        if let Some(start) = self.knob_press_start.lock().unwrap().take() {
                            let duration_ms = start.elapsed().as_millis() as u64;
                            if duration_ms >= KNOB_HOLD_THRESHOLD_MS {
                                let _ = self.event_tx.send(QuakeEvent::KnobHold { duration_ms });
                            } else {
                                // Brief tap — treat as press
                                let _ = self.event_tx.send(QuakeEvent::Press { value: 0 });
                            }
                        }
                    } else {
                        // Regular press event
                        *self.knob_press_start.lock().unwrap() = None;
                        let _ = self.event_tx.send(QuakeEvent::Press { value: v });
                    }
                }
                _ => {}
            }
        }

        if report.op_code == RESP_STATE {
            // Resolve any pending query waiting on this cmdID.
            {
                let mut waiters = self.waiters.lock().unwrap();
                let mut i = 0;
                while i < waiters.len() {
                    if waiters[i].cmd_id == report.cmd_id {
                        let w = waiters.remove(i);
                        let _ = w.tx.send(report.clone());
                        // keep index; remove shifted next element into place
                        continue;
                    }
                    i += 1;
                }
            }

            // Update cached state + emit a typed event.
            let event = match report.cmd_id {
                CTL_INFO => {
                    let name = *report.sub_data.first().unwrap_or(&0);
                    let version = match report.sub_data.get(1..4) {
                        Some(v) => format!("{}.{}.{}", v[0], v[1], v[2]),
                        None => "0.0.0".to_string(),
                    };
                    {
                        let mut s = self.state.lock().unwrap();
                        s.device_name = Some(name);
                        s.version = Some(version.clone());
                    }
                    Some(QuakeEvent::Info {
                        device_name: name,
                        version,
                    })
                }
                CTL_BRIGHTNESS => {
                    let v = *report.sub_data.first().unwrap_or(&0);
                    self.state.lock().unwrap().brightness = Some(v);
                    Some(QuakeEvent::Brightness { value: v })
                }
                CTL_MIC => {
                    let on = report.sub_data.first().copied().unwrap_or(0) == 1;
                    self.state.lock().unwrap().mic = Some(on);
                    Some(QuakeEvent::Mic { enabled: on })
                }
                CTL_KEEP_ALIVE => Some(QuakeEvent::Pong),
                0 => {
                    let success = report.sub_data.first().copied().unwrap_or(0) == 0x90;
                    Some(QuakeEvent::Result { success })
                }
                _ => None,
            };
            if let Some(ev) = event {
                let _ = self.event_tx.send(ev);
            }
        }
    }

    fn handle_control_error(&self) {
        // The control read errored — drop the handle; the loop will rebind.
        *self.control.lock().unwrap() = None;
        self.state.lock().unwrap().connected = false;
        let _ = self.event_tx.send(QuakeEvent::Disconnected);
    }

    /// Try to open the control device. Returns true on success.
    fn rebind_control(&self) -> bool {
        let opened = {
            let api = match self.api.lock() {
                Ok(a) => a,
                Err(_) => return false,
            };
            match find_control_device(&api) {
                Some(info) => match api.open_path(info.path()) {
                    Ok(d) => Some(d),
                    Err(_) => None,
                },
                None => None,
            }
        };

        match opened {
            Some(dev) => {
                // Best-effort: keep reads time-bounded even if we set blocking.
                let _ = dev.set_blocking_mode(false);
                *self.control.lock().unwrap() = Some(dev);
                self.state.lock().unwrap().connected = true;
                let _ = self.event_tx.send(QuakeEvent::Connected);
                true
            }
            None => false,
        }
    }

    fn touch_loop(&self) {
        let mut buf = [0u8; 64];
        loop {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            let needs_rebind = self.touch.lock().unwrap().is_none();
            if needs_rebind && !self.rebind_touch() {
                thread::sleep(Duration::from_secs(2));
                continue;
            }

            let outcome = {
                let mut guard = self.touch.lock().unwrap();
                match guard.as_mut() {
                    Some(d) => match d.read_timeout(&mut buf, 100) {
                        Ok(0) => TouchOutcome::Empty,
                        Ok(n) => TouchOutcome::Data(buf[..n].to_vec()),
                        Err(_) => TouchOutcome::Error,
                    },
                    None => TouchOutcome::Empty,
                }
            };

            match outcome {
                TouchOutcome::Data(bytes) => {
                    if let Some(points) = decode_touch_raw(&bytes) {
                        self.note_interaction();
                        let _ = self.event_tx.send(QuakeEvent::Touch { points });
                    }
                }
                TouchOutcome::Empty => {}
                TouchOutcome::Error => {
                    *self.touch.lock().unwrap() = None;
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    fn rebind_touch(&self) -> bool {
        let opened = {
            let api = match self.api.lock() {
                Ok(a) => a,
                Err(_) => return false,
            };
            match find_touch_device(&api) {
                Some(info) => match api.open_path(info.path()) {
                    Ok(d) => Some(d),
                    Err(_) => None,
                },
                None => None,
            }
        };
        match opened {
            Some(d) => {
                let _ = d.set_blocking_mode(false);
                *self.touch.lock().unwrap() = Some(d);
                true
            }
            None => false,
        }
    }
}

impl Drop for QuakeDevice {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

enum ReadOutcome {
    Data(Vec<u8>),
    Empty,
    Error,
}

enum TouchOutcome {
    Data(Vec<u8>),
    Empty,
    Error,
}

// ---- Device discovery ------------------------------------------------------

/// Find the first QUAKE control interface, preferring one whose product string
/// is literally "QUAKE".
pub fn find_control_device(api: &HidApi) -> Option<&DeviceInfo> {
    let mut fallback = None;
    for d in api.device_list() {
        if d.usage() == QUAKE_USAGE && d.usage_page() == QUAKE_USAGE_PAGE {
            if d.product_string() == Some(QUAKE_PRODUCT_NAME) {
                return Some(d);
            }
            fallback.get_or_insert(d);
        }
    }
    fallback
}

/// Find the QUAKE touch HID interface, if connected.
pub fn find_touch_device(api: &HidApi) -> Option<&DeviceInfo> {
    api.device_list().find(|d| {
        d.vendor_id() == TOUCH_VID
            && d.product_id() == TOUCH_PID
            && d.usage() == TOUCH_USAGE
            && d.usage_page() == TOUCH_USAGE_PAGE
    })
}
