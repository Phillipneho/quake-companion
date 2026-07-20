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
  let swipeStartX: number | null = null;
  let swipeLastX = 0;
  const SWIPE_THRESHOLD = 220;

  $effect(() => {
    const pts = $touchPoints;
    if (pts.length > 0) {
      const x = pts[0].x;
      if (swipeStartX === null) swipeStartX = x;
      swipeLastX = x;
    } else if (swipeStartX !== null) {
      const dx = swipeLastX - swipeStartX;
      if (Math.abs(dx) > SWIPE_THRESHOLD) {
        movePanel(dx > 0 ? -1 : 1);
      }
      swipeStartX = null;
    }
  });

  // --- Lifecycle ------------------------------------------------------------
  onMount(async () => {
    await startDeviceEvents();
    try {
      await api.wake();
    } catch {
      /* no device — handled by overlay */
    }
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
      class="waiting-overlay absolute inset-0 z-50 flex items-center justify-center bg-[#0a0b0e]/85 backdrop-blur-sm"
    >
      <div class="flex flex-col items-center gap-6 text-center">
        <!-- Scanning line animation -->
        <div class="scan-frame relative h-16 w-48 overflow-hidden">
          <div class="scan-line absolute left-0 top-0 h-full w-px bg-[#00d9ff]"></div>
          <div class="absolute inset-0 border border-[rgba(0,217,255,0.08)]"></div>
        </div>
        <div class="flex flex-col items-center gap-1.5">
          <p class="font-display text-sm font-medium tracking-[0.3em] uppercase text-[#e8eef2]/80">
            Waiting for device
          </p>
          <p class="font-data text-xs text-[#6b7785]">
            Connect the DK-QUAKE panel · HID 0x61 / 0xFF60
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .panels-viewport {
    /* leaves room for StatusBar (top) + PanelIndicator (bottom) */
  }

  .scan-line {
    animation: scan-sweep 2.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }

  @keyframes scan-sweep {
    0% {
      left: 0%;
      opacity: 0;
    }
    10% {
      opacity: 1;
    }
    90% {
      opacity: 1;
    }
    100% {
      left: 100%;
      opacity: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .scan-line {
      animation: none;
      left: 50%;
      opacity: 0.5;
    }
  }
</style>