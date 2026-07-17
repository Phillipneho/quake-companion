<script lang="ts">
  import { deviceState } from "../stores/device";
  import { Wifi, WifiOff, Sun, SunDim } from "lucide-svelte";

  let now = $state(new Date());

  $effect(() => {
    const id = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(id);
  });

  const timeStr = $derived(
    now.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    }),
  );

  // Brightness as 0..1 for the sun icon fill cue.
  const brightPct = $derived(
    $deviceState.brightness == null
      ? null
      : Math.round(($deviceState.brightness / 255) * 100),
  );
</script>

<header
  class="status-bar absolute left-0 right-0 top-0 z-40 flex h-11 items-center justify-between px-6 text-sm text-white/80"
>
  <div class="flex items-center gap-3">
    <span class="font-mono text-quake-glow tracking-widest">QUAKE</span>
    <span class="text-white/30">|</span>
    {#if $deviceState.connected}
      <span class="flex items-center gap-1.5 text-quake">
        <Wifi size={16} />
        <span class="text-xs">connected</span>
        {#if $deviceState.version}
          <span class="text-white/40">v{$deviceState.version}</span>
        {/if}
      </span>
    {:else}
      <span class="flex items-center gap-1.5 text-white/40">
        <WifiOff size={16} />
        <span class="text-xs">no device</span>
      </span>
    {/if}
  </div>

  <div class="flex items-center gap-4">
    {#if brightPct !== null}
      <span class="flex items-center gap-1.5 text-white/70">
        {#if brightPct > 50}
          <Sun size={16} class="text-quake" />
        {:else}
          <SunDim size={16} />
        {/if}
        <span class="font-mono text-xs tabular-nums">{brightPct}%</span>
      </span>
    {/if}
    <span class="font-mono text-base tabular-nums text-white">{timeStr}</span>
  </div>
</header>

<style>
  .status-bar {
    /* faint top hairline */
    border-bottom: 1px solid rgba(0, 217, 255, 0.08);
    background: linear-gradient(to bottom, rgba(5, 7, 10, 0.9), rgba(5, 7, 10, 0));
  }
</style>