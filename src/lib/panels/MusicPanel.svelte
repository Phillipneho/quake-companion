<script lang="ts">
  import { Play, Pause, SkipForward, SkipBack, Volume2, Music2 } from "lucide-svelte";

  // Placeholder now-playing. Wire to a real source (MPRIS / Spotify API /
  // OpenClaw) later.
  let playing = $state(false);
  let progress = $state(0.42); // 0..1
  let elapsed = $state(93); // seconds
  const duration = 214;

  $effect(() => {
    if (!playing) return;
    const id = setInterval(() => {
      elapsed = Math.min(duration, elapsed + 1);
      progress = elapsed / duration;
    }, 1000);
    return () => clearInterval(id);
  });

  function fmt(s: number): string {
    const m = Math.floor(s / 60);
    const sec = String(Math.floor(s % 60)).padStart(2, "0");
    return `${m}:${sec}`;
  }
</script>

<section class="panel flex h-full w-full items-center gap-10 px-16">
  <!-- Album art -->
  <div class="art flex h-44 w-44 shrink-0 items-center justify-center rounded-2xl bg-gradient-to-br from-quake/40 to-ink-700">
    <Music2 size={56} class="text-white/80" />
  </div>

  <div class="flex min-w-0 flex-1 flex-col">
    <div class="text-xs uppercase tracking-widest text-quake/70">Now playing</div>
    <div class="truncate text-4xl font-light text-white">Midnight Drive</div>
    <div class="truncate text-xl text-white/60">Neon District · Synthwave Vol. III</div>

    <!-- Progress -->
    <div class="mt-4 flex items-center gap-3">
      <span class="w-12 text-right font-mono text-sm tabular-nums text-white/60">{fmt(elapsed)}</span>
      <div class="relative h-1.5 flex-1 rounded-full bg-white/10">
        <div
          class="absolute left-0 top-0 h-full rounded-full bg-quake"
          style="width: {progress * 100}%; box-shadow: 0 0 8px rgba(0,217,255,0.6);"
        ></div>
      </div>
      <span class="w-12 font-mono text-sm tabular-nums text-white/60">{fmt(duration)}</span>
    </div>

    <!-- Controls -->
    <div class="mt-5 flex items-center gap-6">
      <button class="touch-target rounded-full p-2 text-white/80 hover:text-quake" aria-label="Previous">
        <SkipBack size={28} />
      </button>
      <button
        class="play-btn touch-target flex items-center justify-center rounded-full bg-quake text-ink-950"
        aria-label={playing ? "Pause" : "Play"}
        onclick={() => (playing = !playing)}
      >
        {#if playing}
          <Pause size={30} fill="currentColor" />
        {:else}
          <Play size={30} fill="currentColor" />
        {/if}
      </button>
      <button class="touch-target rounded-full p-2 text-white/80 hover:text-quake" aria-label="Next">
        <SkipForward size={28} />
      </button>
      <div class="ml-auto flex items-center gap-2 text-white/60">
        <Volume2 size={20} />
        <div class="h-1 w-28 rounded-full bg-white/10">
          <div class="h-full w-2/3 rounded-full bg-white/70"></div>
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .art {
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
  animation: float 6s ease-in-out infinite;
  }
  .play-btn {
    width: 64px;
    height: 64px;
    box-shadow: 0 0 20px rgba(0, 217, 255, 0.45);
  }
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
  }
  @media (prefers-reduced-motion: reduce) {
    .art { animation: none; }
  }
</style>