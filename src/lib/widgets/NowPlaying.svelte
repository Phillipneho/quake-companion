<script lang="ts">
  // Now Playing — Spotify music controller.
  // Phase 1.5 will implement Spotify Web API with OAuth PKCE.
  // For now, a styled placeholder.

  let playing = $state(false);
  let title = $state("No track loaded");
  let artist = $state("—");
</script>

<section class="now-playing flex h-full w-full flex-col justify-center gap-3 px-6">
  <span class="label-track text-[#3a8b9e]">Now Playing</span>
  <div class="flex items-center gap-4">
    <div class="album-art flex h-16 w-16 items-center justify-center rounded bg-[#1a1d26] border border-[rgba(255,255,255,0.04)]">
      <span class="font-data text-xs text-[#6b7785]">♪</span>
    </div>
    <div class="flex flex-col gap-1">
      <span class="font-display text-base text-[#e8eef2] truncate">{title}</span>
      <span class="font-display text-sm text-[#6b7785] truncate">{artist}</span>
    </div>
  </div>
  <div class="transport flex items-center gap-4 mt-1">
    <button class="btn" onclick={() => playing = !playing}>
      {playing ? "❚❚" : "▶"}
    </button>
    <button class="btn">⏮</button>
    <button class="btn">⏭</button>
  </div>
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
  .btn { background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); color: #e8eef2; padding: 4px 12px; border-radius: 4px; font-size: 14px; cursor: pointer; }
  .btn:hover { background: rgba(0,217,255,0.08); }
</style>