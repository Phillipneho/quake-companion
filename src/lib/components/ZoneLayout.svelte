<script lang="ts">
  // ZoneLayout — renders the zones for the current page from config.
  // Each zone renders its assigned widget component. Touch focuses a zone.

  import {
    currentZones,
    focusedZone,
    focusZone,
    widgetComponents,
  } from "../stores/layout";

  let { pageIndex = 0 } = $props();

  // Touch handling: focus the zone that was touched
  function handleTouch(e: TouchEvent) {
    const touch = e.touches[0];
    if (!touch) return;
    const x = touch.clientX;
    const zones = $currentZones;
    for (let i = 0; i < zones.length; i++) {
      if (x >= zones[i].x && x < zones[i].x + zones[i].width) {
        focusZone(i);
        return;
      }
    }
    focusZone(-1);
  }
</script>

<svelte:window on:touchstart={handleTouch} />

<div
  class="zone-container relative h-full w-full overflow-hidden"
  style="width: 1920px;"
>
  {#each $currentZones as zone, i (zone.id)}
    <div
      class="zone absolute top-0 bottom-0 transition-all duration-200"
      style="left: {zone.x}px; width: {zone.width}px;"
      class:focused={$focusedZone === i}
      class:dimmed={$focusedZone >= 0 && $focusedZone !== i}
    >
      {#if zone.widget && widgetComponents[zone.widget]}
        {@const Widget = widgetComponents[zone.widget]}
        <Widget />
      {:else if zone.widget}
        <div class="flex h-full w-full items-center justify-center">
          <span class="font-data text-xs text-[#6b7785]">{zone.widget}</span>
        </div>
      {:else}
        <div class="flex h-full w-full items-center justify-center">
          <span class="font-data text-xs text-[#6b7785]/50">Empty zone</span>
        </div>
      {/if}
    </div>
  {/each}

  <!-- Zone dividers (hairline) -->
  {#each $currentZones.slice(0, -1) as zone, i (zone.id)}
    <div
      class="divider pointer-events-none absolute top-4 bottom-4"
      style="left: {zone.x + zone.width - 0.5}px; width: 1px;"
    ></div>
  {/each}
</div>

<style>
  .zone {
    padding: 0;
  }

  .zone.focused {
    box-shadow: inset 0 0 0 1px rgba(0, 217, 255, 0.2);
  }

  .zone.dimmed {
    opacity: 0.55;
  }

  .divider {
    background: rgba(255, 255, 255, 0.04);
  }
</style>