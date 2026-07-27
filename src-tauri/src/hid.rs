//! QUAKE raw-HID protocol — pure framing/parsing, no I/O.
//!
//! Ported from the shipping DK-Suite JS reference
//! (template/modules/quake-device/protocol.js). The 0xA3 "short command"
//! family is used over the device's raw-HID interface.
//!
//! Outgoing frame (HID report, leading report-id 0x00 prepended):
//!   [0x00, 0xA3, payloadLen+1, flag, ...payload, checksum]
//!   checksum = (flag + sum(payload)) % 0xFF
//!
//! Incoming report (report-id already stripped by hidapi):
//!   [0xA3, len, opCode, cmdID, ...subData, checksum]
//!   checksum = sum(opCode, cmdID, ...subData) % 0xFF

use serde::Serialize;

// ---- Command family / flags -------------------------------------------------

/// Outgoing control-frame marker (first byte of the frame body).
pub const CMD_CONTROL: u8 = 0xA3;

/// Frame flag (third byte of an outgoing control frame).
/// 1 = set/action (fire and forget), 2 = query / keep-alive ping.
pub const FLAG_SET: u8 = 1;
pub const FLAG_QUERY: u8 = 2;

// ---- Control payload command ids (first byte of the payload) ----------------

pub const CTL_BUZZER: u8 = 2; // [2, tone] piezo tone (0 = silent)
pub const CTL_MIC: u8 = 3; // [3, on?1:0] set ; [3] query
pub const CTL_SCREEN: u8 = 4; // [4, on?1:0]
pub const CTL_BRIGHTNESS: u8 = 5; // [5, 0..255] set ; [5] query
pub const CTL_LED: u8 = 6; // [6, mode] WS2812 ring effect (0=off,1/2/3=on)
pub const CTL_KEY: u8 = 16; // [16, action, identifier, keyCode]
pub const CTL_INFO: u8 = 46; // [46] query -> name + firmware version
pub const CTL_DFU: u8 = 47; // [47, 3] enter atmel-dfu bootloader
pub const CTL_KEEP_ALIVE: u8 = 239; // [239] watchdog ping (flag 2)

// ---- Incoming report opCodes ------------------------------------------------

pub const RESP_KNOB: u8 = 3; // jog wheel: cmdID 1 = rotate, 2 = press
pub const RESP_STATE: u8 = 0x55; // state report: cmdID identifies the value
pub const TOUCH_CMD_ID: u8 = 26; // touch report cmdID

// ---- HID device match criteria ----------------------------------------------

// The touch panel is a SEPARATE USB HID device (vendor "hotlotus", not the
// QUAKE's own VID). Match by [vendorId, productId, usage, usagePage].
pub const TOUCH_VID: u16 = 1810;
pub const TOUCH_PID: u16 = 16;
pub const TOUCH_USAGE: u16 = 0x71; // 113
pub const TOUCH_USAGE_PAGE: u16 = 0xFF73; // 65395

// The QUAKE control endpoint is the VIA raw-HID interface.
pub const QUAKE_USAGE: u16 = 0x61;
pub const QUAKE_USAGE_PAGE: u16 = 0xFF60;
pub const QUAKE_PRODUCT_NAME: &str = "QUAKE";

/// Firmware blanks the display if it stops receiving keep-alive pings.
pub const KEEP_ALIVE_MS: u64 = 15_000;
/// Companion pings every 1500ms to keep the backlight awake reliably.
pub const KEEP_ALIVE_INTERVAL_MS: u64 = 1_500;

// ---- Idle power management defaults ----------------------------------------
/// Dim to this brightness level after the idle-dim timeout elapses.
pub const DIM_BRIGHTNESS_DEFAULT: u8 = 30;
/// Seconds of inactivity before dimming the screen.
pub const IDLE_DIM_SECS_DEFAULT: u64 = 30;
/// Seconds of inactivity before turning the screen off entirely.
 pub const IDLE_SLEEP_SECS_DEFAULT: u64 = 120;
/// Brightness restored on wake.
 pub const WAKE_BRIGHTNESS_DEFAULT: u8 = 255;

// ---- Knob hold gesture thresholds ------------------------------------------
/// Minimum press duration (ms) before classifying as a hold.
 pub const KNOB_HOLD_THRESHOLD_MS: u64 = 500;
/// Value the firmware reportedly sends at the start of a hold gesture.
 pub const KNOB_HOLD_START: u8 = 5;
/// Value the firmware reportedly sends at the end of a hold gesture.
 pub const KNOB_HOLD_END: u8 = 0xFF;

/// A parsed incoming control report (report-id already stripped).
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub op_code: u8,
    pub cmd_id: u8,
    pub sub_data: Vec<u8>,
}

/// A single decoded touch point. Coordinates are in the panel's native pixel
/// space (1920x480). Y is reported bottom-up by the firmware.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TouchPoint {
    pub action: u8,
    pub x: u16,
    pub y: u16,
}

/// Build a 0xA3 control frame body (without the leading report-id byte).
/// `checksum = (flag + sum(payload)) % 0xFF`.
pub fn wrap_control(payload: &[u8], flag: u8) -> Vec<u8> {
    let mut body = Vec::with_capacity(payload.len() + 4);
    body.push(CMD_CONTROL);
    body.push((payload.len() + 1) as u8);
    body.push(flag);
    let mut checksum: u16 = flag as u16;
    for &b in payload {
        body.push(b);
        checksum += b as u16;
    }
    body.push((checksum % 0xFF) as u8);
    body
}

/// Wrap a control frame into a writable HID report (prepends report-id 0x00).
pub fn to_report(payload: &[u8], flag: u8) -> Vec<u8> {
    let mut r = Vec::with_capacity(payload.len() + 5);
    r.push(0x00);
    r.extend(wrap_control(payload, flag));
    r
}

/// Parse an incoming control report. Accepts a byte slice with or without a
/// leading report-id (0x00) byte. Returns `None` if it is not a valid 0xA3
/// frame or the checksum does not validate.
pub fn parse_report(data: &[u8]) -> Option<Report> {
    // Some platforms prepend the report-id byte; skip a leading 0x00.
    let start = match data.first() {
        Some(&b) if b == CMD_CONTROL => 0,
        _ if data.get(1) == Some(&CMD_CONTROL) => 1,
        _ => return None,
    };

    let e = &data[start..];
    if e.len() < 4 {
        return None;
    }
    let len = e[1] as usize;
    // e = [0xA3, len, opCode, cmdID, ...subData, checksum]
    if e.len() < 2 + len + 1 {
        return None;
    }

    let mut sum: u16 = 0;
    for i in 0..len {
        sum += e[2 + i] as u16;
    }
    if (sum % 0xFF) as u8 != e[2 + len] {
        return None;
    }

    let op_code = e[2];
    let cmd_id = e[3];
    let sub_end = 4 + len.saturating_sub(2);
    let sub_data = if sub_end <= e.len() {
        e[4..sub_end].to_vec()
    } else {
        e[4..].to_vec()
    };
    Some(Report {
        op_code,
        cmd_id,
        sub_data,
    })
}

/// Decode a touch report straight from the touch device's bytes (with or
/// without a leading report-id). Unlike control reports these are NOT
/// checksum-validated; the product only checks `bytes[0]==0xA3 && bytes[3]==26`.
/// Returns an array of `{ action, x, y }` points, or `None`.
pub fn decode_touch_raw(data: &[u8]) -> Option<Vec<TouchPoint>> {
    let start = match data.first() {
        Some(&b) if b == CMD_CONTROL => 0,
        _ if data.get(1) == Some(&CMD_CONTROL) => 1,
        _ => return None,
    };

    let a = &data[start..];
    if a.len() < 5 {
        return None;
    }
    if a[3] != TOUCH_CMD_ID {
        return None;
    }

    let count = a[4] as usize;
    let mut points = Vec::with_capacity(count);
    for n in 0..count {
        let o = 5 + 5 * n;
        if a.len() < o + 5 {
            break;
        }
        let action = a[o];
        let x = ((a[o + 4] as u16) << 8) | (a[o + 3] as u16);
        let y = ((a[o + 2] as u16) << 8) | (a[o + 1] as u16);
        points.push(TouchPoint { action, x, y });
    }
    Some(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_a_set_frame() {
        // [5, 200] brightness set, flag SET (1)
        let report = to_report(&[CTL_BRIGHTNESS, 200], FLAG_SET);
        // leading report id + 0xA3 + len + flag + payload + checksum
        assert_eq!(report[0], 0x00);
        assert_eq!(report[1], CMD_CONTROL);
        assert_eq!(report[2], 3); // payload.len()+1
        assert_eq!(report[3], FLAG_SET);
        assert_eq!(report[4], CTL_BRIGHTNESS);
        assert_eq!(report[5], 200);
        // checksum = (1 + 5 + 200) % 255 = 206
        assert_eq!(report[6], ((FLAG_SET as u16 + 5 + 200) % 0xFF) as u8);
    }

    #[test]
    fn parses_a_state_report() {
        // Build a STATE(0x55) brightness report: cmdID=5, subData=[128].
        let op = RESP_STATE;
        let cmd = CTL_BRIGHTNESS;
        let sub = [128u8];
        // frame: [0xA3, len, opCode, cmdID, ...sub, checksum]
        // len counts opCode..end-of-subData (i.e. 2 + sub.len())
        let body_len = (2 + sub.len()) as u8;
        let mut frame = vec![CMD_CONTROL, body_len, op, cmd];
        frame.extend(sub);
        let sum = (op as u16 + cmd as u16 + sub.iter().map(|b| *b as u16).sum::<u16>()) % 0xFF;
        frame.push(sum as u8);

        let r = parse_report(&frame).expect("should parse");
        assert_eq!(r.op_code, RESP_STATE);
        assert_eq!(r.cmd_id, CTL_BRIGHTNESS);
        assert_eq!(r.sub_data, vec![128]);
    }

    #[test]
    fn decodes_two_touch_points() {
        // [0xA3, len, op, 26, count=2, p1..(5), p2..(5)]
        let p1 = [1u8, 0, 0, 0x80, 0x07]; // action=1, x=0x0780=1920, y=0
        let p2 = [0u8, 0x20, 0x03, 0x00, 0x04]; // action=0, x=0x0400=1024, y=0x0320=800
        let count = 2u8;
        let mut sub = vec![count];
        sub.extend(p1);
        sub.extend(p2);
        let body_len = (2 + sub.len()) as u8;
        let mut frame = vec![CMD_CONTROL, body_len, 0x00, TOUCH_CMD_ID];
        frame.extend(sub);
        // checksum not checked by decode_touch_raw, but include something
        frame.push(0x00);
        let pts = decode_touch_raw(&frame).expect("should decode");
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].action, 1);
        assert_eq!(pts[0].x, 1920);
        assert_eq!(pts[1].x, 1024);
        assert_eq!(pts[1].y, 800);
    }
}
