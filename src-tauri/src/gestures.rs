//! Touch gesture recognition — swipe, long-press, two-finger drag, edge swipe.
//!
//! The gesture engine processes raw touch points from the QUAKE panel's
//! touch HID device and classifies them into high-level gestures that
//! the frontend can react to.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// ---- Gesture types ----------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Gesture {
    /// Swipe left (→ next page)
    SwipeLeft,
    /// Swipe right (→ prev page)
    SwipeRight,
    /// Long press on a zone (→ context menu)
    LongPress { x: u32, y: u32 },
    /// Two-finger drag (→ rearrange zones)
    TwoFingerDrag { dx: i32, dy: i32 },
    /// Edge swipe from left (→ settings)
    EdgeSwipeLeft,
    /// Edge swipe from right (→ settings)
    EdgeSwipeRight,
    /// Double tap (→ back to page selector)
    DoubleTap { x: u32, y: u32 },
    /// Tap (→ focus zone)
    Tap { x: u32, y: u32 },
}

// ---- Gesture engine --------------------------------------------------------

/// Configuration thresholds for gesture detection.
#[derive(Clone, Debug)]
pub struct GestureConfig {
    /// Minimum swipe distance in pixels.
    pub swipe_threshold: u32,
    /// Long press duration in milliseconds.
    pub long_press_ms: u64,
    /// Edge zone width in pixels (swipe from edge = edge swipe).
    pub edge_width: u32,
    /// Double-tap max interval in milliseconds.
    pub double_tap_ms: u64,
    /// Maximum tap movement to still count as a tap.
    pub tap_move_threshold: u32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            swipe_threshold: 150,
            long_press_ms: 600,
            edge_width: 40,
            double_tap_ms: 300,
            tap_move_threshold: 20,
        }
    }
}

/// Internal state for tracking an in-progress touch sequence.
struct TouchTrack {
    start_x: u32,
    start_y: u32,
    start_time: Instant,
    last_x: u32,
    last_y: u32,
    finger_count: usize,
    moved: bool,
}

/// The gesture engine processes touch events and emits classified gestures.
pub struct GestureEngine {
    config: GestureConfig,
    track: Option<TouchTrack>,
    last_tap: Option<(u32, u32, Instant)>,
}

impl GestureEngine {
    pub fn new() -> Self {
        Self {
            config: GestureConfig::default(),
            track: None,
            last_tap: None,
        }
    }

    pub fn set_config(&mut self, config: GestureConfig) {
        self.config = config;
    }

    /// Process a touch event. Call with the raw touch points (action, x, y).
    /// Action 1 = touch down, 0 = lift.
    pub fn process(&mut self, points: &[(u8, u32, u32)]) -> Option<Gesture> {
        if points.is_empty() {
            // All fingers lifted — classify the gesture
            return self.classify();
        }

        let now = Instant::now();

        if self.track.is_none() {
            // New touch sequence
            let first = points[0];
            self.track = Some(TouchTrack {
                start_x: first.1,
                start_y: first.2,
                start_time: now,
                last_x: first.1,
                last_y: first.2,
                finger_count: points.len(),
                moved: false,
            });
        } else {
            // Update existing track
            if let Some(t) = &mut self.track {
                t.last_x = points[0].1;
                t.last_y = points[0].2;
                t.finger_count = points.len();
                let dx = (t.last_x as i32 - t.start_x as i32).abs() as u32;
                let dy = (t.last_y as i32 - t.start_y as i32).abs() as u32;
                if dx > self.config.tap_move_threshold || dy > self.config.tap_move_threshold {
                    t.moved = true;
                }
            }
        }

        None
    }

    fn classify(&mut self) -> Option<Gesture> {
        let track = self.track.take()?;
        let dx = track.last_x as i32 - track.start_x as i32;
        let dy = track.last_y as i32 - track.start_y as i32;
        let elapsed = track.start_time.elapsed();
        let abs_dx = dx.unsigned_abs();
        let abs_dy = dy.unsigned_abs();

        // Two-finger drag
        if track.finger_count >= 2 && track.moved {
            return Some(Gesture::TwoFingerDrag { dx, dy });
        }

        // Edge swipe (starts within edge zone)
        if track.start_x < self.config.edge_width && abs_dx > self.config.swipe_threshold {
            return Some(Gesture::EdgeSwipeLeft);
        }
        let display_width = 1920u32;
        if track.start_x > display_width - self.config.edge_width && abs_dx > self.config.swipe_threshold {
            return Some(Gesture::EdgeSwipeRight);
        }

        // Swipe (horizontal, fast enough)
        if abs_dx > self.config.swipe_threshold && abs_dx > abs_dy {
            return Some(if dx < 0 { Gesture::SwipeLeft } else { Gesture::SwipeRight });
        }

        // Long press (no significant movement, held long enough)
        if !track.moved && elapsed >= Duration::from_millis(self.config.long_press_ms) {
            return Some(Gesture::LongPress { x: track.start_x, y: track.start_y });
        }

        // Double tap detection
        if !track.moved && elapsed < Duration::from_millis(self.config.long_press_ms) {
            if let Some((lx, ly, lt)) = self.last_tap {
                let dist = ((track.start_x as i32 - lx as i32).abs() + (track.start_y as i32 - ly as i32).abs()) as u32;
                if dist < self.config.tap_move_threshold && lt.elapsed() < Duration::from_millis(self.config.double_tap_ms) {
                    self.last_tap = None;
                    return Some(Gesture::DoubleTap { x: track.start_x, y: track.start_y });
                }
            }
            self.last_tap = Some((track.start_x, track.start_y, Instant::now()));
            return Some(Gesture::Tap { x: track.start_x, y: track.start_y });
        }

        None
    }
}

impl Default for GestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Burn-in prevention ----------------------------------------------------

/// Burn-in prevention: periodic pixel-shift and brightness scheduling.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurnInConfig {
    /// Pixel-shift interval in seconds (0 = disabled).
    #[serde(default = "default_shift_interval")]
    pub shift_interval_secs: u64,
    /// Pixel-shift amount in pixels.
    #[serde(default = "default_shift_amount")]
    pub shift_amount: u32,
    /// Time-based brightness schedule: [(hour, brightness 0-255)].
    /// If empty, no schedule (use idle management only).
    #[serde(default)]
    pub brightness_schedule: Vec<(u32, u8)>,
    /// Minimum brightness during scheduled dimming.
    #[serde(default = "default_min_brightness")]
    pub min_brightness: u8,
}

fn default_shift_interval() -> u64 { 300 }
fn default_shift_amount() -> u32 { 2 }
fn default_min_brightness() -> u8 { 10 }

impl Default for BurnInConfig {
    fn default() -> Self {
        Self {
            shift_interval_secs: 300,
            shift_amount: 2,
            brightness_schedule: vec![
                (6, 255),   // 6 AM: full brightness
                (22, 30),   // 10 PM: dim
                (0, 10),    // Midnight: minimum
            ],
            min_brightness: 10,
        }
    }
}

/// Get the target brightness for a given hour from the schedule.
pub fn scheduled_brightness(config: &BurnInConfig, hour: u32) -> u8 {
    if config.brightness_schedule.is_empty() {
        return 255;
    }

    let mut sorted = config.brightness_schedule.clone();
    sorted.sort_by_key(|(h, _)| *h);

    // Find the last schedule entry with hour <= current hour
    let mut brightness = config.min_brightness;
    for (h, b) in &sorted {
        if *h <= hour {
            brightness = *b;
        } else {
            break;
        }
    }
    brightness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesture_swipe_left() {
        let mut engine = GestureEngine::new();
        // Touch down at x=960
        engine.process(&[(1, 960, 240)]);
        // Move to x=200 (swipe left)
        engine.process(&[(1, 200, 240)]);
        // Lift
        let gesture = engine.process(&[]);
        assert_eq!(gesture, Some(Gesture::SwipeLeft));
    }

    #[test]
    fn gesture_swipe_right() {
        let mut engine = GestureEngine::new();
        engine.process(&[(1, 200, 240)]);
        engine.process(&[(1, 960, 240)]);
        let gesture = engine.process(&[]);
        assert_eq!(gesture, Some(Gesture::SwipeRight));
    }

    #[test]
    fn gesture_edge_swipe_left() {
        let mut engine = GestureEngine::new();
        // Start at x=10 (within edge zone)
        engine.process(&[(1, 10, 240)]);
        engine.process(&[(1, 500, 240)]);
        let gesture = engine.process(&[]);
        assert_eq!(gesture, Some(Gesture::EdgeSwipeLeft));
    }

    #[test]
    fn burn_in_config_defaults() {
        let cfg = BurnInConfig::default();
        assert!(cfg.shift_interval_secs > 0);
        assert!(!cfg.brightness_schedule.is_empty());
    }

    #[test]
    fn scheduled_brightness_daytime() {
        let cfg = BurnInConfig::default();
        assert_eq!(scheduled_brightness(&cfg, 10), 255);
    }

    #[test]
    fn scheduled_brightness_evening() {
        let cfg = BurnInConfig::default();
        assert_eq!(scheduled_brightness(&cfg, 22), 30);
    }

    #[test]
    fn scheduled_brightness_morning_before_six() {
        let cfg = BurnInConfig::default();
        assert_eq!(scheduled_brightness(&cfg, 3), 10);
    }

    #[test]
    fn scheduled_brightness_empty_schedule() {
        let cfg = BurnInConfig { brightness_schedule: vec![], min_brightness: 10, ..Default::default() };
        assert_eq!(scheduled_brightness(&cfg, 15), 255);
    }
}