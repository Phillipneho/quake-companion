// Svelte stores for device + system state, plus panel navigation and live
// touch data. All Tauri IPC is wrapped here so the panels stay declarative.

import { writable, derived, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Layout navigation imported for knob event routing.
import { movePage, togglePageSelector, focusZone } from "./layout";

// ---- Types -----------------------------------------------------------------

export type PowerState = "awake" | "dim" | "sleep";

export interface DeviceState {
  connected: boolean;
  brightness: number | null;
  mic: boolean | null;
  led: boolean;
  version: string | null;
  device_name: number | null;
  power_state: PowerState;
}

export interface PowerConfig {
  idle_dim_secs: number;
  idle_sleep_secs: number;
  dim_brightness: number;
  wake_brightness: number;
}

// ---- VIA RGB ring types ----------------------------------------------------

export type RgbEffect =
  | "off"
  | "plain"
  | "breathe"
  | "rainbow"
  | "swirl"
  | "snake"
  | "knight"
  | "xmas"
  | "gradient"
  | "rainbow_test";

export interface HsvColor {
  hue: number;
  sat: number;
}

export interface RgbColor {
  r: number;
  g: number;
  b: number;
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

// ---- Config types ----------------------------------------------------------

export interface Settings {
  idle_dim_secs: number;
  idle_sleep_secs: number;
  dim_brightness: number;
  wake_brightness: number;
  ring_effect: RgbEffect;
  ring_brightness: number;
  ring_color_1: [number, number];
  ring_color_2: [number, number];
  ring_persist: boolean;
  keep_alive_ms: number;
  mic_enabled: boolean;
  clock_24h: boolean;
  timezone: string;
}

export interface Zone {
  id: string;
  x: number;
  width: number;
  widget: string | null;
}

export interface Page {
  name: string;
  label: string;
  zones: Zone[];
}

export interface Profile {
  name: string;
  label: string;
  pages: string[];
  ring_effect: RgbEffect | null;
}

export interface Config {
  schema_version: number;
  settings: Settings;
  pages: Page[];
  profiles: Profile[];
  active_profile: string;
}

// Tagged event from the Rust `QuakeEvent` enum (serde tag = "type").
export type QuakeEvent =
  | { type: "Connected" }
  | { type: "Disconnected" }
  | { type: "Rotate"; data: { direction: number } }
  | { type: "Press"; data: { value: number } }
  | { type: "KnobHold"; data: { duration_ms: number } }
  | { type: "Touch"; data: { points: TouchPoint[] } }
  | { type: "Info"; data: { device_name: number; version: string } }
  | { type: "Brightness"; data: { value: number } }
  | { type: "Mic"; data: { enabled: boolean } }
  | { type: "Pong" }
  | { type: "Result"; data: { success: boolean } }
  | { type: "PowerStateChanged"; data: { state: PowerState } };

// ---- Stores ----------------------------------------------------------------

export const deviceState = writable<DeviceState>({
  connected: false,
  brightness: null,
  mic: null,
  led: false,
  version: null,
  device_name: null,
  power_state: "awake",
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
  // Power management
  dimScreen: () => call<void>("dim_screen"),
  sleepScreen: () => call<void>("sleep_screen"),
  wakeScreen: () => call<void>("wake_screen"),
  getPowerState: () => call<PowerState>("get_power_state"),
  setPowerConfig: (config: PowerConfig) => call<void>("set_power_config", { config }),
  getPowerConfig: () => call<PowerConfig>("get_power_config"),
  // VIA RGB ring control
  viaSetEffect: (effect: RgbEffect) => call<void>("via_set_effect", { effect }),
  viaSetBrightness: (brightness: number) => call<void>("via_set_brightness", { brightness }),
  viaSetColor1: (hue: number, sat: number) => call<void>("via_set_color_1", { hue, sat }),
  viaSetColor2: (hue: number, sat: number) => call<void>("via_set_color_2", { hue, sat }),
  viaSetCustomColor: (index: number, hue: number, sat: number) => call<void>("via_set_custom_color", { index, hue, sat }),
  viaSaveLighting: () => call<void>("via_save_lighting"),
  viaEepromReset: () => call<void>("via_eeprom_reset"),
  viaBootloaderJump: () => call<void>("via_bootloader_jump"),
  // Config system
  getConfig: () => call<Config>("get_config"),
  saveConfig: (config: Config) => call<void>("save_config", { newConfig: config }),
  resetConfig: () => call<Config>("reset_config"),
  setActiveProfile: (profile: string) => call<void>("set_active_profile", { profile }),
  // Spotify
  spotifyAuthStatus: () => call<boolean>("spotify_auth_status"),
  spotifyStartAuth: () => call<string>("spotify_start_auth"),
  spotifyExchangeCode: (code: string, codeVerifier: string) => call<void>("spotify_exchange_code", { code, codeVerifier }),
  spotifyLogout: () => call<void>("spotify_logout"),
  spotifyNowPlaying: () => call<any>("spotify_now_playing"),
  spotifyTogglePlay: () => call<void>("spotify_toggle_play"),
  spotifyNext: () => call<void>("spotify_next"),
  spotifyPrevious: () => call<void>("spotify_previous"),
  spotifySetVolume: (volume: number) => call<void>("spotify_set_volume", { volume }),
  spotifyGetState: () => call<any>("spotify_get_state"),
  // Notifications
  getNotifications: () => call<any[]>("get_notifications"),
  getUnreadNotifications: () => call<any[]>("get_unread_notifications"),
  getUnreadCount: () => call<number>("get_unread_count"),
  markNotificationRead: (id: string) => call<void>("mark_notification_read", { id }),
  snoozeNotification: (id: string) => call<void>("snooze_notification", { id }),
  dismissNotification: (id: string) => call<void>("dismiss_notification", { id }),
  fetchGithubNotifications: () => call<number>("fetch_github_notifications"),
  // Home Assistant
  haGetStates: () => call<any[]>("ha_get_states"),
  haGetEntity: (entityId: string) => call<any>("ha_get_entity", { entityId }),
  haToggle: (entityId: string) => call<void>("ha_toggle", { entityId }),
  haTurnOn: (entityId: string) => call<void>("ha_turn_on", { entityId }),
  haTurnOff: (entityId: string) => call<void>("ha_turn_off", { entityId }),
  haSetBrightness: (entityId: string, brightness: number) => call<void>("ha_set_brightness", { entityId, brightness }),
  haMediaPlayPause: (entityId: string) => call<void>("ha_media_play_pause", { entityId }),
  haSetVolume: (entityId: string, volume: number) => call<void>("ha_set_volume", { entityId, volume }),
  // OpenClaw Panel
  ocGetMessages: () => call<any[]>("oc_get_messages"),
  ocClearConversation: () => call<void>("oc_clear_conversation"),
  ocSendText: (text: string) => call<void>("oc_send_text", { text }),
  ocTranscribeAndRespond: (audioData: number[], format: string) => call<string>("oc_transcribe_and_respond", { audioData, format }),
  ocRespondToText: (text: string) => call<string>("oc_respond_to_text", { text }),
  ocGetRecordingState: () => call<string>("oc_get_recording_state"),
  // AI-Composed Panels
  composePanel: (prompt: string) => call<any>("compose_panel", { prompt }),
  saveComposedPanel: (panel: any) => call<void>("save_composed_panel", { panel }),
};

// ---- Config store ---------------------------------------------------------

// Re-export config from the dedicated config store to avoid circular imports.
export { config } from "./config";

/** Load config from backend on startup. */
export async function loadConfig(): Promise<void> {
  try {
    const cfg = await api.getConfig();
    config.set(cfg);
  } catch (e) {
    console.error("Failed to load config:", e);
  }
}

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
      // Rotate = page navigation (page selector handles its own highlight)
      movePage(ev.data.direction >= 0 ? 1 : -1);
      lastKnobAt.set(Date.now());
      break;
    case "Press":
      // Knob press = toggle zone focus
      focusZone(-1);
      lastKnobAt.set(Date.now());
      break;
    case "KnobHold":
      // Knob hold = open/close page selector
      togglePageSelector();
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
    case "PowerStateChanged":
      deviceState.update((s) => ({ ...s, power_state: ev.data.state }));
      break;
    default:
      break;
  }
}