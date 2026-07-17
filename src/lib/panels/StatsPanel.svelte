<script lang="ts">
  import { systemStats } from "../stores/device";
  import { Cpu, MemoryStick, HardDrive, Activity } from "lucide-svelte";

  // Rolling CPU history for a mini sparkline.
  const HISTORY = 60;
  let cpuHistory = $state<number[]>(Array(HISTORY).fill(0));

  $effect(() => {
    const v = $systemStats.cpu_usage;
    cpuHistory = [...cpuHistory.slice(1), v];
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

  // Sparkline path for the CPU history (0..100 -> viewBox 100x30).
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
</script>

<section class="panel flex h-full w-full items-center gap-10 px-16">
  <!-- CPU -->
  <div class="metric flex flex-1 items-center gap-4">
    <Cpu size={36} class="text-quake" />
    <div class="flex flex-col">
      <div class="text-xs uppercase tracking-widest text-white/40">CPU</div>
      <div class="text-4xl font-light tabular-nums text-white">
        {pct($systemStats.cpu_usage)}
      </div>
    </div>
    <svg viewBox="0 0 100 30" preserveAspectRatio="none" class="h-9 w-40">
      <path d={sparkPath} fill="none" stroke="#00d9ff" stroke-width="1.2" />
    </svg>
  </div>

  <!-- Memory -->
  <div class="metric flex flex-1 items-center gap-4">
    <MemoryStick size={36} class="text-quake" />
    <div class="flex flex-col">
      <div class="text-xs uppercase tracking-widest text-white/40">Memory</div>
      <div class="text-4xl font-light tabular-nums text-white">
        {pct($systemStats.memory_usage)}
      </div>
      <div class="text-xs text-white/40">
        {fmtBytes($systemStats.memory_used)} / {fmtBytes($systemStats.memory_total)}
      </div>
    </div>
  </div>

  <!-- Disk -->
  <div class="metric flex flex-1 items-center gap-4">
    <HardDrive size={36} class="text-quake" />
    <div class="flex flex-col">
      <div class="text-xs uppercase tracking-widest text-white/40">Disk</div>
      <div class="text-4xl font-light tabular-nums text-white">
        {pct($systemStats.disk_usage)}
      </div>
      <div class="text-xs text-white/40">
        {fmtBytes($systemStats.disk_used)} / {fmtBytes($systemStats.disk_total)}
      </div>
    </div>
  </div>

  <!-- Network + uptime -->
  <div class="metric flex flex-1 flex-col gap-2">
    <div class="flex items-center gap-3">
      <Activity size={28} class="text-quake" />
      <div>
        <div class="text-xs uppercase tracking-widest text-white/40">Network</div>
        <div class="text-sm tabular-nums text-white">
          ↓ {fmtBytes($systemStats.network_rx)}
          <span class="mx-1 text-white/30">|</span>
          ↑ {fmtBytes($systemStats.network_tx)}
        </div>
      </div>
    </div>
    <div class="text-xs uppercase tracking-widest text-white/40">Uptime</div>
    <div class="text-2xl font-light tabular-nums text-white">
      {fmtUptime($systemStats.uptime)}
    </div>
  </div>
</section>

<style>
  .metric {
    border-left: 2px solid rgba(0, 217, 255, 0.25);
    padding-left: 1.25rem;
  }
</style>