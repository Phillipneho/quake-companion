<script lang="ts">
  import { systemStats } from "../stores/device";
  import { untrack } from "svelte";
  import { Cpu, MemoryStick, HardDrive, Activity } from "lucide-svelte";

  // Rolling CPU history for sparkline.
  const HISTORY = 60;
  let cpuHistory = $state<number[]>(Array(HISTORY).fill(0));

  // Use untrack to prevent the $effect from re-triggering when cpuHistory
  // (which we write to) is read elsewhere — avoids infinite loops.
  $effect(() => {
    const v = $systemStats.cpu_usage;
    untrack(() => {
      cpuHistory = [...cpuHistory.slice(1), v];
    });
  });

  function pct(n: number): string {
    return n.toFixed(1) + "%";
  }

  function fmtBytes(b: number): string {
    if (b >= 1e9) return (b / 1e9).toFixed(1) + " GB";
    if (b >= 1e6) return (b / 1e6).toFixed(1) + " MB";
    if (b >= 1e3) return (b / 1e3).toFixed(1) + " KB";
    return b + " B";
  }

  function fmtUptime(s: number): string {
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    if (d > 0) return `${d}d ${h}h ${m}m`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  // Sparkline path for the CPU history (0..100 -> viewBox 120x40).
  const sparkPath = $derived.by(() => {
    const w = 120;
    const h = 40;
    const step = w / (HISTORY - 1);
    return cpuHistory
      .map((v, i) => {
        const x = i * step;
        const y = h - (Math.min(100, v) / 100) * h;
        return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  });

  const sparkArea = $derived.by(() => {
    return sparkPath + ` L120,40 L0,40 Z`;
  });
</script>

<section class="panel flex h-full w-full items-center gap-8 px-16">
  <!-- CPU — featured with sparkline -->
  <div class="metric flex flex-[1.4] flex-col gap-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Cpu size={18} class="text-[#3a8b9e]" />
        <span class="label-track text-[#6b7785]">CPU</span>
      </div>
      <span class="font-data text-3xl font-300 tabular-nums text-[#ffffff]">
        {pct($systemStats.cpu_usage)}
      </span>
    </div>
    <div class="sparkline-wrap relative">
      <svg viewBox="0 0 120 40" preserveAspectRatio="none" class="h-10 w-full">
        <defs>
          <linearGradient id="sparkGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#00d9ff" stop-opacity="0.15" />
            <stop offset="100%" stop-color="#00d9ff" stop-opacity="0" />
          </linearGradient>
        </defs>
        <path d={sparkArea} fill="url(#sparkGrad)" />
        <path d={sparkPath} fill="none" stroke="#00d9ff" stroke-width="1" />
      </svg>
    </div>
  </div>

  <div class="divider"></div>

  <!-- Memory -->
  <div class="metric flex flex-1 flex-col gap-2">
    <div class="flex items-center gap-2">
      <MemoryStick size={18} class="text-[#3a8b9e]" />
      <span class="label-track text-[#6b7785]">Memory</span>
    </div>
    <span class="font-data text-3xl font-300 tabular-nums text-[#ffffff]">
      {pct($systemStats.memory_usage)}
    </span>
    <span class="font-data text-xs text-[#6b7785]">
      {fmtBytes($systemStats.memory_used)} / {fmtBytes($systemStats.memory_total)}
    </span>
  </div>

  <div class="divider"></div>

  <!-- Disk -->
  <div class="metric flex flex-1 flex-col gap-2">
    <div class="flex items-center gap-2">
      <HardDrive size={18} class="text-[#3a8b9e]" />
      <span class="label-track text-[#6b7785]">Disk</span>
    </div>
    <span class="font-data text-3xl font-300 tabular-nums text-[#ffffff]">
      {pct($systemStats.disk_usage)}
    </span>
    <span class="font-data text-xs text-[#6b7785]">
      {fmtBytes($systemStats.disk_used)} / {fmtBytes($systemStats.disk_total)}
    </span>
  </div>

  <div class="divider"></div>

  <!-- Network + uptime -->
  <div class="metric flex flex-1 flex-col gap-2">
    <div class="flex items-center gap-2">
      <Activity size={18} class="text-[#3a8b9e]" />
      <span class="label-track text-[#6b7785]">Network</span>
    </div>
    <div class="font-data text-sm tabular-nums text-[#e8eef2]">
      ↓ {fmtBytes($systemStats.network_rx)}
      <span class="mx-1 text-[#6b7785]/50">|</span>
      ↑ {fmtBytes($systemStats.network_tx)}
    </div>
    <div class="mt-1 label-track text-[#6b7785]">Uptime</div>
    <div class="font-data text-xl font-300 tabular-nums text-[#e8eef2]">
      {fmtUptime($systemStats.uptime)}
    </div>
  </div>
</section>

<style>
  .metric {
    padding: 0 0.5rem;
  }
  .divider {
    width: 1px;
    height: 60%;
    background: rgba(255, 255, 255, 0.04);
  }
  .sparkline-wrap {
    margin-top: -4px;
  }
</style>