// Svelte stores for device + system state, plus panel navigation and live
// touch data. All Tauri IPC is wrapped here so the panels stay declarative.

import { writable, derived, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ---- Types -----------------------------------------------------------------

export interface DeviceState {
  connected: boolean;
  brightness: number | null;
  mic: boolean | null;
  led: boolean;
  version: string | null;
  device_name: number | null;
}

export interface SystemStats {
  cpu_usage: number;
  memory_usage: number;
  memory_used: number;
  memory_total: number;
  disk_usage: number;
  disk_used: number;
  disk_total: number;
  network_rx: number;
  network_tx: number;
  uptime: number;
}

export interface TouchPoint {
  action: number;
  x: number;
  y: number;
}

// Tagged event from the Rust `QuakeEvent` enum (serde tag = "type").
export type QuakeEvent =
  | { type: "Connected" }
  | { type: "Disconnected" }
  | { type: "Rotate"; data: { direction: number } }
  | { type: "Press"; data: { value: number } }
  | { type: "Touch"; data: { points: TouchPoint[] } }
  | { type: "Info"; data: { device_name: number; version: string } }
  | { type: "Brightness"; data: { value: number } }
  | { type: "Mic"; data: { enabled: boolean } }
  | { type: "Pong" }
  | { type: "Result"; data: { success: boolean } };

// ---- Stores ----------------------------------------------------------------

export const deviceState = writable<DeviceState>({
  connected: false,
  brightness: null,
  mic: null,
  led: false,
  version: null,
  device_name: null,
});

export const systemStats = writable<SystemStats>({
  cpu_usage: 0,
  memory_usage: 0,
  memory_used: 0,
  memory_total: 0,
  disk_usage: 0,
  disk_used: 0,
  disk_total: 0,
  network_rx: 0,
  network_tx: 0,
  uptime: 0,
});

/** Index of the currently visible panel. */
export const activePanel = writable<number>(0);

/** Live touch points (most recent report); cleared on lift. */
export const touchPoints = writable<TouchPoint[]>([]);

/** Last knob event timestamp, for UI feedback flashes. */
export const lastKnobAt = writable<number>(0);

// ---- Panel registry --------------------------------------------------------

export interface PanelMeta {
  id: string;
  label: string;
}

export const panels: PanelMeta[] = [
  { id: "clock", label: "Clock" },
  { id: "stats", label: "Stats" },
  { id: "ai", label: "AI" },
  { id: "shortcuts", label: "Shortcuts" },
  { id: "music", label: "Music" },
  { id: "notifications", label: "Notifs" },
];

export const panelCount = panels.length;

/** Derived: the active panel meta. */
export const activePanelMeta: Readable<PanelMeta> = derived(
  activePanel,
  ($i) => panels[clamp($i, 0, panelCount - 1)],
);

function clamp(n: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, n));
}

/** Move the active panel by `delta` (-1 prev, +1 next), wrapping. */
export function movePanel(delta: number): void {
  activePanel.update((i) => (i + delta + panelCount) % panelCount);
}

// ---- IPC wrappers ----------------------------------------------------------

async function call<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(name, args);
}

export const api = {
  wake: () => call<void>("wake_device"),
  setBrightness: (value: number) => call<void>("set_brightness", { value }),
  setScreen: (on: boolean) => call<void>("set_screen", { on }),
  setMic: (on: boolean) => call<void>("set_mic", { on }),
  setLed: (mode: number) => call<void>("set_led", { mode }),
  ping: () => call<void>("ping_device"),
  getDeviceInfo: () => call<{ device_name: number; version: string }>("get_device_info"),
  getDeviceState: () => call<DeviceState>("get_device_state"),
  getSystemStats: () => call<SystemStats>("get_system_stats"),
};

// ---- Event subscription ----------------------------------------------------

let eventUnlisten: UnlistenFn | null = null;

/** Subscribe to `quake://event` and dispatch to the stores. Idempotent. */
export async function startDeviceEvents(): Promise<void> {
  if (eventUnlisten) return;
  eventUnlisten = await listen<QuakeEvent>("quake://event", (e) => {
    handleEvent(e.payload);
  });
}

export async function stopDeviceEvents(): Promise<void> {
  if (eventUnlisten) {
    await eventUnlisten();
    eventUnlisten = null;
  }
}

function handleEvent(ev: QuakeEvent): void {
  switch (ev.type) {
    case "Connected":
      deviceState.update((s) => ({ ...s, connected: true }));
      // Refresh cached state on (re)connect.
      void api.getDeviceState().then((s) => deviceState.set(s));
      void api.getDeviceInfo().then((info) =>
        deviceState.update((s) => ({
          ...s,
          version: info.version,
          device_name: info.device_name,
        })),
      ).catch(() => {});
      break;
    case "Disconnected":
      deviceState.update((s) => ({ ...s, connected: false }));
      break;
    case "Rotate":
      movePanel(ev.data.direction >= 0 ? 1 : -1);
      lastKnobAt.set(Date.now());
      break;
    case "Press":
      // Knob press = enter / toggle. Panels that care can watch lastKnobAt;
      // here it just nudges brightness as a demonstration affordance.
      lastKnobAt.set(Date.now());
      break;
    case "Touch":
      touchPoints.set(ev.data.points);
      break;
    case "Info":
      deviceState.update((s) => ({
        ...s,
        version: ev.data.version,
        device_name: ev.data.device_name,
      }));
      break;
    case "Brightness":
      deviceState.update((s) => ({ ...s, brightness: ev.data.value }));
      break;
    case "Mic":
      deviceState.update((s) => ({ ...s, mic: ev.data.enabled }));
      break;
    default:
      break;
  }
}