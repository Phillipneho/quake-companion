# Quake Companion

A standalone desktop app for the **Decokee QUAKE** display panel, built for **Windows ARM64** (Surface Pro 11 / Snapdragon X Elite).

The official DK-Suite is x64-only and won't run on ARM. This replaces it with a native Tauri (Rust + Svelte) app that talks directly to the Quake hardware over USB HID.

## Architecture

- **Backend:** Rust + Tauri v2 + `hidapi` — direct HID protocol, no Electron
- **Frontend:** Svelte 5 + Vite + Tailwind CSS — rendered to the 1920×480 display
- **Protocol:** Ported from the open-source [xls/decokee-quake-template](https://github.com/xls/decokee-quake-template) reference

### Dashboard Panels (rotate via knob or swipe)

1. **Clock** — Time, date, weather placeholder
2. **System Stats** — CPU, RAM, disk usage (real-time)
3. **AI Chat** — Quick questions to any OpenAI-compatible endpoint
4. **Shortcuts** — Customizable one-tap actions
5. **Music** — Now playing placeholder
6. **Notifications** — Recent messages placeholder

### Input

- **Knob rotate:** Switch panels (left/right)
- **Knob press:** Select/confirm
- **Touch:** Direct interaction + swipe to navigate panels

## Build Instructions

### Prerequisites

1. **Rust** — install via [rustup](https://rustup.rs):
   ```powershell
   winget install Rustlang.Rustup
   ```
   Then add the ARM64 target:
   ```powershell
   rustup target add aarch64-pc-windows-msvc
   ```

2. **Node.js 20+** — [nodejs.org](https://nodejs.org) (ARM64 build)
   ```powershell
   winget install OpenJS.NodeJS.LTS
   ```

3. **Visual Studio Build Tools** — C++ workload for MSVC:
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
   Select "Desktop development with C++" in the installer.

4. **Tauri CLI:**
   ```powershell
   npm install -g @tauri-apps/cli@latest
   ```

### Build

```powershell
cd quake-companion
npm install
npm run build           # build frontend (Svelte/Vite)
npx tauri build         # build Rust backend + bundle
```

This produces an installer in `src-tauri/target/release/bundle/`.

### Dev Mode

For development with hot reload:
```powershell
npm install
npx tauri dev
```

### Cross-compile from x64 Windows (optional)

If you have an x64 Windows machine and want to build for ARM64:
```powershell
rustup target add aarch64-pc-windows-msvc
npx tauri build --target aarch64-pc-windows-msvc
```

## Project Structure

```
quake-companion/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point (feature-gated)
│   │   ├── lib.rs          # Module exports
│   │   ├── hid.rs          # QUAKE raw-HID protocol (framing, parsing, touch)
│   │   ├── device.rs       # QuakeDevice wrapper (screen, brightness, knob, touch)
│   │   ├── commands.rs     # Tauri IPC commands (frontend callable)
│   │   ├── stats.rs        # System stats (CPU/RAM/disk via sysinfo)
│   │   └── app.rs          # Tauri app setup + event bridging
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # Svelte frontend
│   ├── App.svelte          # Main shell (panel navigation, touch/knob handling)
│   ├── main.ts             # Svelte entry
│   ├── app.html / app.css  # HTML shell + Tailwind
│   ├── lib/
│   │   ├── stores/device.ts    # Svelte stores (device state, stats, panels)
│   │   ├── panels/              # Dashboard panels (6)
│   │   └── components/          # StatusBar, PanelIndicator
│   └── static/
├── package.json
├── vite.config.ts
├── svelte.config.js
└── tailwind.config.js
```

## Protocol Reference

The QUAKE uses a raw HID protocol with the 0xA3 "short command" family:

**Outgoing frame:**
```
[0x00, 0xA3, payloadLen+1, flag, ...payload, checksum]
checksum = (flag + sum(payload)) % 0xFF
```
- flag=1: set/action (fire-and-forget)
- flag=2: query (expects 0x55 state response) + keep-alive ping

**Incoming report:**
```
[0xA3, len, opCode, cmdID, ...subData, checksum]
```

| Command ID | Function    | Payload                     |
|------------|-------------|----------------------------|
| 2          | Buzzer      | [2, tone] (0=silent)       |
| 3          | Mic         | [3, on?1:0] set / [3] query |
| 4          | Screen      | [4, on?1:0]                |
| 5          | Brightness  | [5, 0..255] set / [5] query|
| 6          | LED         | [6, mode] (0=off, 1/2/3=on)|
| 16         | Key         | [16, action, id, keyCode]  |
| 46         | Info        | [46] query → name + fw ver |
| 47         | DFU         | [47, 3] enter bootloader    |
| 239        | Keep-alive  | [239] watchdog ping         |

**Critical:** Display blanks after ~15s without keep-alive ping. The app pings every 14s.

**Touch device** is a separate USB HID device (VID=1810, PID=16, usage=0x71, usagePage=0xFF73). Touch reports use cmdID=26 with 5 bytes per point: `[action, yLo, yHi, xLo, xHi]`. Y origin is bottom-up.

## Hardware

- **Display:** 8.88" IPS LCD, 1920×480, HDMI input
- **Touch:** 5-point capacitive (separate USB HID)
- **Knob:** Infinite rotary encoder + push button + RGB ring
- **Mic:** Far-field with noise reduction
- **Power:** 5V/2A via USB-C
- **Connection:** HDMI (video) + USB-C (data/power)

## Credits

- Protocol ported from [xls/decokee-quake-template](https://github.com/xls/decokee-quake-template)
- Original protocol from [DecoKeeAI/DecoKeeAI](https://github.com/DecoKeeAI/DecoKeeAI)
- Built with [Tauri v2](https://tauri.app), [Svelte 5](https://svelte.dev), [hidapi](https://github.com/ruabmbua/hidapi-rs)

## License

MIT