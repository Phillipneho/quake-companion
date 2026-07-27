<script lang="ts">
  // System Monitor — CPU, RAM, disk, network, uptime.
  // Reuses the existing StatsPanel logic but as a zone widget.

  import { systemStats } from "../stores/device";
  import { untrack } from "svelte";

  const HISTORY = 60;
  let cpuHistory = $state<number[]>(Array(HISTORY).fill(0));

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
    if (d > 0) return `${d}d ${h}h`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  const sparkPath = $derived.by(() => {
    const w = 100;
    const h = 30;
    const step = w / (HISTORY - 1);
    return cpuHistory
      .map((v, i) => {
        const x = i * step;
        const y = h - (Math.min(100, v) / 100) * h;
        return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  });

  const sparkArea = $derived(sparkPath + ` L100,30 L0,30 Z`);
</script>

<section class="sysmon flex h-full w-full flex-col justify-center gap-3 px-6">
  <!-- CPU with sparkline -->
  <div class="metric-row flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="label-track text-[#6b7785]">CPU</span>
    </div>
    <span class="font-data text-2xl font-300 tabular-nums text-white">
      {pct($systemStats.cpu_usage)}
    </span>
  </div>
  <svg viewBox="0 0 100 30" preserveAspectRatio="none" class="h-8 w-full">
    <defs>
      <linearGradient id="sysmonGrad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#00d9ff" stop-opacity="0.12" />
        <stop offset="100%" stop-color="#00d9ff" stop-opacity="0" />
      </linearGradient>
    </defs>
    <path d={sparkArea} fill="url(#sysmonGrad)" />
    <path d={sparkPath} fill="none" stroke="#00d9ff" stroke-width="0.8" />
  </svg>

  <div class="metric-row flex items-center justify-between">
    <span class="label-track text-[#6b7785]">RAM</span>
    <span class="font-data text-lg font-300 tabular-nums text-[#e8eef2]">
      {pct($systemStats.memory_usage)}
    </span>
  </div>
  <div class="metric-row flex items-center justify-between">
    <span class="label-track text-[#6b7785]">Disk</span>
    <span class="font-data text-lg font-300 tabular-nums text-[#e8eef2]">
      {pct($systemStats.disk_usage)}
    </span>
  </div>
  <div class="metric-row flex items-center justify-between">
    <span class="label-track text-[#6b7785]">Net</span>
    <span class="font-data text-sm tabular-nums text-[#6b7785]">
      ↓{fmtBytes($systemStats.network_rx)} ↑{fmtBytes($systemStats.network_tx)}
    </span>
  </div>
  <div class="metric-row flex items-center justify-between">
    <span class="label-track text-[#6b7785]">Uptime</span>
    <span class="font-data text-sm text-[#e8eef2]">
      {fmtUptime($systemStats.uptime)}
    </span>
  </div>
</section>

<style>
  .metric-row {
    padding: 2px 0;
  }
  .label-track {
    letter-spacing: 0.18em;
    text-transform: uppercase;
    font-size: 0.625rem;
    font-weight: 500;
  }
</style>