<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  import {
    activePanel,
    deviceState,
    movePanel,
    panels,
    startDeviceEvents,
    stopDeviceEvents,
    systemStats,
    touchPoints,
    api,
  } from "./lib/stores/device";

  import StatusBar from "./lib/components/StatusBar.svelte";
  import PanelIndicator from "./lib/components/PanelIndicator.svelte";
  import ClockPanel from "./lib/panels/ClockPanel.svelte";
  import StatsPanel from "./lib/panels/StatsPanel.svelte";
  import AIPanel from "./lib/panels/AIPanel.svelte";
  import ShortcutsPanel from "./lib/panels/ShortcutsPanel.svelte";
  import MusicPanel from "./lib/panels/MusicPanel.svelte";
  import NotificationsPanel from "./lib/panels/NotificationsPanel.svelte";

  let statsTimer: ReturnType<typeof setInterval> | undefined;

  // --- Touch swipe navigation -----------------------------------------------
  // The panel sends multi-touch points in native pixels (1920x480). We track
  // the first contact X and, when all fingers lift (empty report), navigate by
  // the horizontal delta if it exceeds the threshold.
  let swipeStartX: number | null = null;
  let swipeLastX = 0;
  const SWIPE_THRESHOLD = 220; // px in panel-native space

  $effect(() => {
    const pts = $touchPoints;
    if (pts.length > 0) {
      const x = pts[0].x;
      if (swipeStartX === null) swipeStartX = x;
      swipeLastX = x;
    } else if (swipeStartX !== null) {
      const dx = swipeLastX - swipeStartX;
      if (Math.abs(dx) > SWIPE_THRESHOLD) {
        movePanel(dx > 0 ? -1 : 1); // swipe right -> previous
      }
      swipeStartX = null;
    }
  });

  // --- Lifecycle ------------------------------------------------------------
  onMount(async () => {
    await startDeviceEvents();
    // Try to wake immediately if a device is present; failures are expected in
    // the no-device dev state and just leave the "waiting" overlay up.
    try {
      await api.wake();
    } catch {
      /* no device — handled by overlay */
    }
    // Refresh system stats every 3s for the Stats panel.
    await refreshStats();
    statsTimer = setInterval(refreshStats, 3000);
  });

  onDestroy(() => {
    if (statsTimer) clearInterval(statsTimer);
    void stopDeviceEvents();
  });

  async function refreshStats(): Promise<void> {
    try {
      const s = await api.getSystemStats();
      systemStats.set(s);
    } catch {
      // Outside Tauri (e.g. `vite dev` in a browser) the IPC call throws.
    }
  }
</script>

<div id="app-root" class="relative h-screen w-screen overflow-hidden">
  <StatusBar />

  <!-- Panel strip: each panel is 100vw wide; translate the strip to reveal the
       active one. Slides horizontally with a 180ms ease. -->
  <div
    class="panels-viewport absolute left-0 right-0 top-11 bottom-8 overflow-hidden"
  >
    <div
      class="panel-strip flex h-full"
      style="transform: translateX({-$activePanel * 100}vw); transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);"
    >
      <div class="panel-cell h-full w-screen shrink-0">
        <ClockPanel />
      </div>
      <div class="panel-cell h-full w-screen shrink-0">
        <StatsPanel />
      </div>
      <div class="panel-cell h-full w-screen shrink-0">
        <AIPanel />
      </div>
      <div class="panel-cell h-full w-screen shrink-0">
        <ShortcutsPanel />
      </div>
      <div class="panel-cell h-full w-screen shrink-0">
        <MusicPanel />
      </div>
      <div class="panel-cell h-full w-screen shrink-0">
        <NotificationsPanel />
      </div>
    </div>
  </div>

  <PanelIndicator />

  {#if !$deviceState.connected}
    <div
      class="waiting-overlay absolute inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    >
      <div class="flex flex-col items-center gap-3 text-center">
        <div class="h-10 w-10 animate-pulse rounded-full border-2 border-quake/60"></div>
        <p class="text-lg font-medium text-quake-glow">Waiting for QUAKE device…</p>
        <p class="text-sm text-white/50">
          Connect the DK-QUAKE panel (control HID usage 0x61/0xFF60).
        </p>
      </div>
    </div>
  {/if}
</div>

<style>
  .panels-viewport {
    /* leaves room for StatusBar (top) + PanelIndicator (bottom) */
  }
</style>