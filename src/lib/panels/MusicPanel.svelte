<script lang="ts">
  import { Play, Pause, SkipForward, SkipBack, Volume2, Music2 } from "lucide-svelte";

  let playing = $state(false);
  let progress = $state(0.42);
  let elapsed = $state(93);
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

<section class="panel flex h-full w-full items-center gap-12 px-16">
  <!-- Album art — elegant, subtle float -->
  <div class="art flex h-40 w-40 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-[#1a1d26] to-[#0a0b0e]" style="border: 1px solid rgba(255,255,255,0.04);">
    <Music2 size={48} class="text-[#3a8b9e]" />
  </div>

  <div class="flex min-w-0 flex-1 flex-col">
    <div class="label-track text-[#3a8b9e]">Now playing</div>
    <div class="truncate font-display text-3xl font-400 text-[#ffffff] mt-1">Midnight Drive</div>
    <div class="truncate font-display text-base text-[#6b7785] mt-0.5">Neon District · Synthwave Vol. III</div>

    <!-- Progress -->
    <div class="mt-5 flex items-center gap-3">
      <span class="w-10 font-data text-xs tabular-nums text-[#6b7785] text-right">{fmt(elapsed)}</span>
      <div class="relative h-[3px] flex-1 rounded-full bg-[rgba(255,255,255,0.06)]">
        <div
          class="absolute left-0 top-0 h-full rounded-full bg-[#00d9ff]"
          style="width: {progress * 100}%; box-shadow: 0 0 6px rgba(0,217,255,0.3);"
        ></div>
      </div>
      <span class="w-10 font-data text-xs tabular-nums text-[#6b7785]">{fmt(duration)}</span>
    </div>

    <!-- Controls -->
    <div class="mt-6 flex items-center gap-5">
      <button class="touch-target rounded-full p-1.5 text-[#6b7785] transition-colors hover:text-[#e8eef2]" aria-label="Previous">
        <SkipBack size={24} />
      </button>
      <button
        class="play-btn touch-target flex items-center justify-center rounded-full transition-transform active:scale-95"
        aria-label={playing ? "Pause" : "Play"}
        onclick={() => (playing = !playing)}
      >
        {#if playing}
          <Pause size={26} fill="currentColor" />
        {:else}
          <Play size={26} fill="currentColor" class="ml-0.5" />
        {/if}
      </button>
      <button class="touch-target rounded-full p-1.5 text-[#6b7785] transition-colors hover:text-[#e8eef2]" aria-label="Next">
        <SkipForward size={24} />
      </button>
      <div class="ml-auto flex items-center gap-2.5">
        <Volume2 size={18} class="text-[#6b7785]" />
        <div class="h-[3px] w-24 rounded-full bg-[rgba(255,255,255,0.06)]">
          <div class="h-full w-2/3 rounded-full bg-[rgba(255,255,255,0.5)]"></div>
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .art {
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    animation: float 6s ease-in-out infinite;
  }
  .play-btn {
    width: 56px;
    height: 56px;
    background: #00d9ff;
    color: #0a0b0e;
    box-shadow: 0 0 16px rgba(0, 217, 255, 0.3);
  }
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-3px); }
  }
  @media (prefers-reduced-motion: reduce) {
    .art { animation: none; }
  }
</style>