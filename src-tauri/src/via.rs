//! QMK VIA protocol — RGB ring / backlight control over the QUAKE's VIA HID interface.
//!
//! The VIA protocol uses raw HID reports (33 bytes, leading report-id 0x00).
//! Commands are sent as `[0x00, command_id, ...payload]` and the device
//! echoes the command bytes back in the response, followed by data.
//!
//! This module talks to the same HID interface as our 0xA3 control protocol
//! (usage 0x61, usagePage 0xFF60) — the QUAKE panel exposes both VIA and the
//! custom 0xA3 protocol on the same endpoint.

use serde::{Deserialize, Serialize};

// ---- VIA API command IDs ----------------------------------------------------

/// Command prefix byte for all VIA commands.
pub const VIA_COMMAND_START: u8 = 0x00;

pub const VIA_GET_PROTOCOL_VERSION: u8 = 0x01;
pub const VIA_GET_KEYBOARD_VALUE: u8 = 0x02;
pub const VIA_SET_KEYBOARD_VALUE: u8 = 0x03;
pub const VIA_BACKLIGHT_CONFIG_SET_VALUE: u8 = 0x07;
pub const VIA_BACKLIGHT_CONFIG_GET_VALUE: u8 = 0x08;
pub const VIA_BACKLIGHT_CONFIG_SAVE: u8 = 0x09;
pub const VIA_EEPROM_RESET: u8 = 0x0A;
pub const VIA_BOOTLOADER_JUMP: u8 = 0x0B;

// ---- Backlight value IDs (used with GET/SET_KEYBOARD_VALUE) ----------------

pub const BACKLIGHT_BRIGHTNESS: u8 = 9;
pub const BACKLIGHT_EFFECT: u8 = 10;
pub const BACKLIGHT_COLOR_1: u8 = 12;
pub const BACKLIGHT_COLOR_2: u8 = 13;
pub const BACKLIGHT_CUSTOM_COLOR: u8 = 23;

// ---- VIA protocol versions --------------------------------------------------

pub const PROTOCOL_ALPHA: u16 = 7;
pub const PROTOCOL_BETA: u16 = 8;
pub const PROTOCOL_GAMMA: u16 = 9;

// ---- RGB effect modes (QMK standard) ---------------------------------------

/// QMK RGB effect modes. The exact set supported depends on the firmware
/// compiled into the QUAKE's controller, but these are the standard QMK modes.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RgbEffect {
    /// RGB off.
    Off = 0,
    /// Solid colour.
    Plain = 1,
    /// Breathing animation.
    Breathe = 2,
    /// Rainbow colour cycle.
    Rainbow = 3,
    /// Swirl animation.
    Swirl = 4,
    /// Snake animation.
    Snake = 5,
    /// Knight rider animation.
    Knight = 6,
    /// Christmas theme.
    Xmas = 7,
    /// Static gradient.
    Gradient = 8,
    /// Rainbow drops test mode.
    RainbowTest = 9,
}

impl Default for RgbEffect {
    fn default() -> Self {
        Self::Off
    }
}

impl RgbEffect {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Off,
            1 => Self::Plain,
            2 => Self::Breathe,
            3 => Self::Rainbow,
            4 => Self::Swirl,
            5 => Self::Snake,
            6 => Self::Knight,
            7 => Self::Xmas,
            8 => Self::Gradient,
            _ => Self::RainbowTest, // fallback for unknown modes
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

// ---- HSV colour type --------------------------------------------------------

/// HSV colour for the RGB ring. Hue 0-255, Saturation 0-255 (firmware scale).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HsvColor {
    pub hue: u8,
    pub sat: u8,
}

impl Default for HsvColor {
    fn default() -> Self {
        Self { hue: 0, sat: 255 }
    }
}

/// RGB colour (0-255 per channel) — convenience for frontend display.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl HsvColor {
    /// Convert HSV (hue 0-255, sat 0-255, val=255) to RGB.
    pub fn to_rgb(self) -> RgbColor {
        let h = self.hue;
        let s = self.sat;

        if s == 0 {
            return RgbColor { r: 0, g: 0, b: 0 };
        }

        let region = h / 43;
        let remainder = (h - region * 43) * 6;
        let p = 0u8;
        let q = (255 - s) / 255; // simplified
        let _ = q;
        let v = 255u8;

        let (r, g, b) = match region {
            0 => (v, (v - (v * remainder) / 255), p),
            1 => ((v - (v * remainder) / 255), v, p),
            2 => (p, v, (v - (v * remainder) / 255)),
            3 => (p, (v - (v * remainder) / 255), v),
            4 => ((v - (v * remainder) / 255), p, v),
            _ => (v, p, (v - (v * remainder) / 255)),
        };

        RgbColor { r, g, b }
    }
}

// ---- VIA frame builder / parser --------------------------------------------

/// Build a VIA HID report (33 bytes, leading report-id 0x00).
/// The VIA protocol sends `[0x00, command, ...args]` padded to 33 bytes.
pub fn via_report(command: u8, payload: &[u8]) -> Vec<u8> {
    let mut report = vec![0u8; 33];
    report[0] = VIA_COMMAND_START;
    report[1] = command;
    for (i, &b) in payload.iter().enumerate() {
        if 2 + i < 33 {
            report[2 + i] = b;
        }
    }
    report
}

/// Build a backlight "set value" command: SET_KEYBOARD_VALUE(3) + value_id + args.
pub fn via_set_backlight(value_id: u8, args: &[u8]) -> Vec<u8> {
    let mut payload = vec![value_id];
    payload.extend_from_slice(args);
    via_report(VIA_SET_KEYBOARD_VALUE, &payload)
}

/// Build a backlight "get value" command: GET_KEYBOARD_VALUE(2) + value_id.
pub fn via_get_backlight(value_id: u8) -> Vec<u8> {
    via_report(VIA_GET_KEYBOARD_VALUE, &[value_id])
}

/// Build a backlight config save command.
pub fn via_save_backlight() -> Vec<u8> {
    via_report(VIA_BACKLIGHT_CONFIG_SAVE, &[])
}

/// Build an EEPROM reset command.
pub fn via_eeprom_reset() -> Vec<u8> {
    via_report(VIA_EEPROM_RESET, &[])
}

/// Build a bootloader jump command.
pub fn via_bootloader_jump() -> Vec<u8> {
    via_report(VIA_BOOTLOADER_JUMP, &[])
}

/// Build a protocol version query.
pub fn via_get_protocol_version() -> Vec<u8> {
    via_report(VIA_GET_PROTOCOL_VERSION, &[])
}

/// Parse a VIA response. Returns (command_id, payload_bytes) where
/// payload is everything after the echoed command bytes.
pub fn parse_via_response(data: &[u8]) -> Option<(u8, Vec<u8>)> {
    if data.len() < 2 {
        return None;
    }
    let command = data[1];
    // The response echoes the command at byte 1; data follows after the
    // command + any sub-command bytes. For simple value queries, the
    // response is: [0x00, cmd, value_id, ...data...].
    let payload = data[2..].to_vec();
    Some((command, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_brightness_set_report() {
        let report = via_set_backlight(BACKLIGHT_BRIGHTNESS, &[128]);
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], VIA_SET_KEYBOARD_VALUE);
        assert_eq!(report[2], BACKLIGHT_BRIGHTNESS);
        assert_eq!(report[3], 128);
        assert_eq!(report.len(), 33);
    }

    #[test]
    fn builds_effect_get_report() {
        let report = via_get_backlight(BACKLIGHT_EFFECT);
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], VIA_GET_KEYBOARD_VALUE);
        assert_eq!(report[2], BACKLIGHT_EFFECT);
    }

    #[test]
    fn builds_color_set_report() {
        // Set color 1: hue=120, sat=255
        let report = via_set_backlight(BACKLIGHT_COLOR_1, &[120, 255]);
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], VIA_SET_KEYBOARD_VALUE);
        assert_eq!(report[2], BACKLIGHT_COLOR_1);
        assert_eq!(report[3], 120);
        assert_eq!(report[4], 255);
    }

    #[test]
    fn builds_save_report() {
        let report = via_save_backlight();
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], VIA_BACKLIGHT_CONFIG_SAVE);
    }

    #[test]
    fn effect_roundtrips() {
        for e in [RgbEffect::Off, RgbEffect::Plain, RgbEffect::Breathe, RgbEffect::Rainbow] {
            let v = e.as_u8();
            assert_eq!(RgbEffect::from_u8(v), e);
        }
    }

    #[test]
    fn hsv_to_rgb_black_when_sat_zero() {
        let c = HsvColor { hue: 0, sat: 0 };
        let rgb = c.to_rgb();
        assert_eq!(rgb.r, 0);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn parses_via_response() {
        // Simulated response: [0x00, 0x02, BACKLIGHT_EFFECT, effect_value=3]
        let data = [0x00, VIA_GET_KEYBOARD_VALUE, BACKLIGHT_EFFECT, 3];
        let (cmd, payload) = parse_via_response(&data).expect("should parse");
        assert_eq!(cmd, VIA_GET_KEYBOARD_VALUE);
        assert_eq!(payload, vec![BACKLIGHT_EFFECT, 3]);
    }
}