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

  const brightPct = $derived(
    $deviceState.brightness == null
      ? null
      : Math.round(($deviceState.brightness / 255) * 100),
  );
</script>

<header
  class="status-bar absolute left-0 right-0 top-0 z-40 flex h-11 items-center justify-between px-6"
>
  <div class="flex items-center gap-3">
    <span class="font-display text-[13px] font-600 tracking-[0.35em] uppercase text-[#e8eef2]">
      QUAKE
    </span>
    <span class="text-[#6b7785]/40">·</span>
    {#if $deviceState.connected}
      <span class="flex items-center gap-1.5">
        <Wifi size={14} class="text-[#00d9ff]" />
        <span class="font-display text-[11px] tracking-wider text-[#3a8b9e]">connected</span>
        {#if $deviceState.version}
          <span class="font-data text-[11px] text-[#6b7785]">v{$deviceState.version}</span>
        {/if}
      </span>
    {:else}
      <span class="flex items-center gap-1.5">
        <WifiOff size={14} class="text-[#6b7785]" />
        <span class="font-display text-[11px] tracking-wider text-[#6b7785]">no device</span>
      </span>
    {/if}
  </div>

  <div class="flex items-center gap-5">
    {#if brightPct !== null}
      <span class="flex items-center gap-1.5">
        {#if brightPct > 50}
          <Sun size={14} class="text-[#00d9ff]" />
        {:else}
          <SunDim size={14} class="text-[#6b7785]" />
        {/if}
        <span class="font-data text-[11px] tabular-nums text-[#e8eef2]">{brightPct}%</span>
      </span>
    {/if}
    <span class="font-data text-[15px] tabular-nums text-[#ffffff]">{timeStr}</span>
  </div>
</header>

<style>
  .status-bar {
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: linear-gradient(to bottom, rgba(10, 11, 14, 0.85), rgba(10, 11, 14, 0));
  }
</style>