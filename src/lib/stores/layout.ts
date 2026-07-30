// Layout engine — config-driven zone rendering and page navigation.
// Replaces the hardcoded panel strip with a dynamic system that reads
// from the Config (pages, zones, widgets) and renders the right components.

import { writable, derived, type Readable } from "svelte/store";
import { api, type Config, type Page, type Zone } from "./device";
import { config } from "./config";

// ---- Navigation state -----------------------------------------------------

/** Index of the active page within the active profile's page list. */
export const activePageIndex = writable<number>(0);

/** Active profile name (kept in sync with config). */
export const activeProfile = writable<string>("work");

/** Whether the page selector overlay is open (triggered by knob double-click). */
export const pageSelectorOpen = writable<boolean>(false);

/** Focused zone index within the current page (for touch focus highlight). */
export const focusedZone = writable<number>(-1);

// ---- Derived state --------------------------------------------------------

/** The active profile's page list, resolved from config. */
export const profilePages: Readable<string[]> = derived(
  [config, activeProfile],
  ([$config, $profile]) => {
    if (!$config) return [];
    const profile = $config.profiles.find((p) => p.name === $profile);
    return profile?.pages ?? [];
  },
);

/** The current page object. */
export const currentPage: Readable<Page | null> = derived(
  [config, profilePages, activePageIndex],
  ([$config, $pages, $idx]) => {
    if (!$config || $pages.length === 0) return null;
    const pageName = $pages[clamp($idx, 0, $pages.length - 1)];
    return $config.pages.find((p) => p.name === pageName) ?? null;
  },
);

/** Zones for the current page. */
export const currentZones: Readable<Zone[]> = derived(
  currentPage,
  ($page) => $page?.zones ?? [],
);

/** Total page count in the active profile. */
export const pageCount: Readable<number> = derived(profilePages, ($pages) => $pages.length);

// ---- Navigation actions ---------------------------------------------------

/** Navigate to the next/previous page (wraps). */
export function movePage(delta: number): void {
  profilePages.subscribe(($pages) => {
    if ($pages.length === 0) return;
    activePageIndex.update((i) => (i + delta + $pages.length) % $pages.length);
  })();
}

/** Jump to a specific page index. */
export function goToPage(index: number): void {
  activePageIndex.set(index);
  pageSelectorOpen.set(false);
}

/** Open or close the page selector overlay. */
export function togglePageSelector(): void {
  pageSelectorOpen.update((o) => !o);
}

/** Focus a zone by index (from touch). -1 = unfocus all. */
export function focusZone(index: number): void {
  focusedZone.update((current) => (current === index ? -1 : index));
}

/** Switch to a different profile. */
export async function switchProfile(name: string): Promise<void> {
  await api.setActiveProfile(name);
  activeProfile.set(name);
  activePageIndex.set(0);
  // Reload config to reflect the change
  const cfg = await api.getConfig();
  config.set(cfg);
}

/** Load config and initialise navigation on startup. */
export async function initLayout(): Promise<void> {
  await loadConfig();
  // Set active profile from config
  config.subscribe(($cfg) => {
    if ($cfg) activeProfile.set($cfg.active_profile);
  })();
}

async function loadConfig(): Promise<void> {
  try {
    const cfg = await api.getConfig();
    config.set(cfg);
  } catch (e) {
    console.error("Failed to load config:", e);
  }
}

// ---- Widget component registry ---------------------------------------------
// Maps widget ids to their Svelte components. Built-in widgets are registered
// here; disk widgets would be loaded dynamically (future).

import FlipClock from "../widgets/FlipClock.svelte";
import Weather from "../widgets/Weather.svelte";
import SystemMonitor from "../widgets/SystemMonitor.svelte";
import NowPlaying from "../widgets/NowPlaying.svelte";
import Notifications from "../widgets/Notifications.svelte";
import OpenClawChat from "../widgets/OpenClawChat.svelte";
import Pomodoro from "../widgets/Pomodoro.svelte";
import Calendar from "../widgets/Calendar.svelte";
import DailyStoic from "../widgets/DailyStoic.svelte";
import WebDashboard from "../widgets/WebDashboard.svelte";
import HomeAssistant from "../widgets/HomeAssistant.svelte";

export const widgetComponents: Record<string, any> = {
  "flip-clock": FlipClock,
  weather: Weather,
  "system-monitor": SystemMonitor,
  "now-playing": NowPlaying,
  notifications: Notifications,
  "openclaw-chat": OpenClawChat,
  pomodoro: Pomodoro,
  calendar: Calendar,
  "daily-stoic": DailyStoic,
  "web-dashboard": WebDashboard,
  "home-assistant": HomeAssistant,
};

// ---- Helpers ---------------------------------------------------------------

function clamp(n: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, n));
}