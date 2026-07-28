<script lang="ts">
  // Now Playing — Spotify music controller widget.
  // Polls the Spotify Web API for current track, renders album art + metadata.
  // Transport controls: play/pause, next, prev. Knob = volume when focused.

  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface NowPlaying {
    is_playing: boolean;
    track_name: string;
    artist_name: string;
    album_name: string;
    album_art_url: string;
    duration_ms: number;
    progress_ms: number;
  }

  interface PlaybackState {
    is_playing: boolean;
    volume: number;
  }

  let nowPlaying = $state<NowPlaying | null>(null);
  let playbackState = $state<PlaybackState | null>(null);
  let authed = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function checkAuth() {
    try {
      authed = await invoke<boolean>("spotify_auth_status");
    } catch {
      authed = false;
    }
  }

  async function fetchNowPlaying() {
    if (!authed) return;
    try {
      const np = await invoke<NowPlaying | null>("spotify_now_playing");
      nowPlaying = np;
      error = null;
    } catch (e: any) {
      error = typeof e === "string" ? e : "Failed to fetch playback";
    }
  }

  async function fetchPlaybackState() {
    if (!authed) return;
    try {
      playbackState = await invoke<PlaybackState>("spotify_get_state");
    } catch {
      // Non-critical
    }
  }

  async function togglePlay() {
    try {
      await invoke("spotify_toggle_play");
      // Refresh after a short delay
      setTimeout(fetchNowPlaying, 300);
    } catch (e: any) {
      error = typeof e === "string" ? e : "Playback control failed";
    }
  }

  async function nextTrack() {
    try {
      await invoke("spotify_next");
      setTimeout(fetchNowPlaying, 300);
    } catch (e: any) {
      error = typeof e === "string" ? e : "Next track failed";
    }
  }

  async function prevTrack() {
    try {
      await invoke("spotify_previous");
      setTimeout(fetchNowPlaying, 300);
    } catch (e: any) {
      error = typeof e === "string" ? e : "Previous track failed";
    }
  }

  async function setVolume(vol: number) {
    try {
      await invoke("spotify_set_volume", { volume: Math.round(vol) });
      if (playbackState) playbackState.volume = Math.round(vol);
    } catch {
      // Non-critical
    }
  }

  // Progress bar
  const progressPct = $derived(
    nowPlaying && nowPlaying.duration_ms > 0
      ? (nowPlaying.progress_ms / nowPlaying.duration_ms) * 100
      : 0,
  );

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, "0")}`;
  }

  onMount(async () => {
    await checkAuth();
    if (authed) {
      await Promise.all([fetchNowPlaying(), fetchPlaybackState()]);
      // Poll now-playing every 3 seconds
      pollTimer = setInterval(fetchNowPlaying, 3000);
    }
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<section class="now-playing flex h-full w-full flex-col justify-center gap-3 px-6">
  {#if !authed}
    <!-- Not authenticated -->
    <div class="flex flex-col items-center gap-3 py-4">
      <span class="label-track text-[#3a8b9e]">Spotify</span>
      <span class="font-display text-sm text-[#6b7785]">Not connected</span>
      <span class="font-data text-xs text-[#6b7785]/60">Auth flow requires setup</span>
    </div>
  {:else if error}
    <div class="flex flex-col items-center gap-2 py-4">
      <span class="label-track text-[#3a8b9e]">Spotify</span>
      <span class="font-display text-sm text-[#6b7785]">{error}</span>
    </div>
  {:else if nowPlaying}
    <!-- Now playing -->
    <div class="flex items-center gap-4">
      <!-- Album art -->
      {#if nowPlaying.album_art_url}
        <img
          src={nowPlaying.album_art_url}
          alt={nowPlaying.album_name}
          class="album-art h-20 w-20 rounded object-cover"
        />
      {:else}
        <div class="album-art-placeholder flex h-20 w-20 items-center justify-center rounded bg-[#1a1d26]">
          <span class="font-data text-2xl text-[#6b7785]">♪</span>
        </div>
      {/if}

      <!-- Track info -->
      <div class="flex flex-1 flex-col gap-1 overflow-hidden">
        <span class="track-name font-display text-base font-500 text-[#e8eef2] truncate">
          {nowPlaying.track_name}
        </span>
        <span class="artist-name font-display text-sm text-[#6b7785] truncate">
          {nowPlaying.artist_name}
        </span>
        <span class="album-name font-display text-xs text-[#6b7785]/70 truncate">
          {nowPlaying.album_name}
        </span>
      </div>

      <!-- Playing indicator -->
      {#if nowPlaying.is_playing}
        <div class="playing-bars flex items-end gap-0.5 h-4">
          <span class="bar bar-1"></span>
          <span class="bar bar-2"></span>
          <span class="bar bar-3"></span>
        </div>
      {/if}
    </div>

    <!-- Progress bar -->
    <div class="progress-row flex items-center gap-2">
      <span class="font-data text-[10px] text-[#6b7785] tabular-nums">
        {fmtTime(nowPlaying.progress_ms)}
      </span>
      <div class="progress-track flex-1 h-0.5 bg-[rgba(255,255,255,0.06)] rounded-full overflow-hidden">
        <div
          class="progress-fill h-full bg-[#00d9ff] rounded-full transition-all duration-1000"
          style="width: {progressPct}%"
        ></div>
      </div>
      <span class="font-data text-[10px] text-[#6b7785] tabular-nums">
        {fmtTime(nowPlaying.duration_ms)}
      </span>
    </div>

    <!-- Transport controls -->
    <div class="transport-row flex items-center gap-3 mt-1">
      <button class="btn" onclick={prevTrack} title="Previous">⏮</button>
      <button class="btn btn-primary" onclick={togglePlay} title="Play/Pause">
        {nowPlaying.is_playing ? "❚❚" : "▶"}
      </button>
      <button class="btn" onclick={nextTrack} title="Next">⏭</button>
      {#if playbackState}
        <div class="volume-row flex items-center gap-2 ml-2">
          <span class="font-data text-[10px] text-[#6b7785]">VOL</span>
          <div class="volume-track w-20 h-0.5 bg-[rgba(255,255,255,0.06)] rounded-full">
            <div
              class="volume-fill h-full bg-[#3a8b9e] rounded-full"
              style="width: {playbackState.volume}%"
            ></div>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- No track playing -->
    <div class="flex flex-col items-center gap-3 py-4">
      <span class="label-track text-[#3a8b9e]">Spotify</span>
      <span class="font-display text-sm text-[#6b7785]">Nothing playing</span>
    </div>
  {/if}
</section>

<style>
  .label-track {
    letter-spacing: 0.18em;
    text-transform: uppercase;
    font-size: 0.625rem;
    font-weight: 500;
  }

  .album-art {
    border: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  .btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #e8eef2;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .btn-primary {
    background: rgba(0, 217, 255, 0.08);
    border-color: rgba(0, 217, 255, 0.15);
    color: #00d9ff;
    padding: 4px 14px;
  }

  .btn-primary:hover {
    background: rgba(0, 217, 255, 0.15);
  }

  /* Playing bars animation */
  .playing-bars .bar {
    width: 2px;
    background: #00d9ff;
    border-radius: 1px;
  }

  .bar-1 { height: 60%; animation: bar-bounce 0.8s ease-in-out infinite; }
  .bar-2 { height: 100%; animation: bar-bounce 0.6s ease-in-out infinite 0.2s; }
  .bar-3 { height: 40%; animation: bar-bounce 0.7s ease-in-out infinite 0.1s; }

  @keyframes bar-bounce {
    0%, 100% { transform: scaleY(0.4); }
    50% { transform: scaleY(1); }
  }
</style>