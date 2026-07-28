<script lang="ts">
  // Web Dashboard — full-screen web view for any URL.
  // HA dashboards, Grafana, etc. Knob scrolls, tap clicks, double-click returns to page selector.
  // Per-page auth injection via custom headers (future).

  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { config } from "../stores/device";

  let url = $state("");
  let loaded = $state(false);
  let error = $state<string | null>(null);
  let iframe: HTMLIFrameElement | null = null;

  // Get URL from config settings or default to HA dashboard
  $effect(() => {
    if ($config?.settings?.ha?.url) {
      url = $config.settings.ha.url + "/dashboard";
    }
  });

  function handleLoad() {
    loaded = true;
    error = null;
  }

  function handleError() {
    error = "Failed to load URL";
    loaded = false;
  }

  // Navigate to a new URL
  function navigate(newUrl: string) {
    url = newUrl;
    loaded = false;
    error = null;
  }

  // Reload the iframe
  function reload() {
    if (iframe) {
      loaded = false;
      iframe.src = url;
    }
  }

  // Double-click returns to page selector (handled by parent)
  function handleDblClick() {
    // Emit a custom event that the parent catches
    window.dispatchEvent(new CustomEvent("quake-back-to-selector"));
  }
</script>

<section
  class="web-dash flex h-full w-full flex-col"
  ondblclick={handleDblClick}
>
  {#if url}
    <div class="url-bar flex items-center gap-2 px-4 py-1.5 bg-[rgba(0,0,0,0.3)]">
      <span class="font-data text-[10px] text-[#6b7785] truncate flex-1">{url}</span>
      <button class="reload-btn" onclick={reload} title="Reload">↻</button>
    </div>
    <div class="iframe-wrap flex-1 relative">
      {#if error}
        <div class="error-state absolute inset-0 flex flex-col items-center justify-center gap-2">
          <span class="font-display text-sm text-[#6b7785]">{error}</span>
          <span class="font-data text-xs text-[#6b7785]/60">{url}</span>
        </div>
      {:else}
        <iframe
          bind:this={iframe}
          src={url}
          class="w-full h-full border-0"
          onload={handleLoad}
          onerror={handleError}
          sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
        ></iframe>
        {#if !loaded}
          <div class="loading-overlay absolute inset-0 flex items-center justify-center bg-[#0a0b0e]">
            <span class="font-data text-xs text-[#6b7785]">Loading…</span>
          </div>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="no-url flex h-full w-full flex-col items-center justify-center gap-3">
      <span class="label-track text-[#3a8b9e]">Web Dashboard</span>
      <span class="font-display text-sm text-[#6b7785]">No URL configured</span>
      <span class="font-data text-xs text-[#6b7785]/60">Set a URL in config to display a web dashboard</span>
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

  .url-bar {
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .reload-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #6b7785;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 12px;
    cursor: pointer;
  }

  .reload-btn:hover {
    color: #e8eef2;
    background: rgba(255, 255, 255, 0.08);
  }

  .loading-overlay {
    z-index: 10;
    transition: opacity 200ms ease;
  }
</style>