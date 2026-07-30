<script lang="ts">
  // PageSelector — overlay shown on knob double-click.
  // Shows all pages in the active profile as a grid. Rotate to highlight,
  // push to switch.

  import {
    pageSelectorOpen,
    profilePages,
    activePageIndex,
    goToPage,
  } from "../stores/layout"
  import { config } from "../stores/device";

  let highlightedIndex = $state(0);

  // Keep highlighted index in sync with active page
  $effect(() => {
    highlightedIndex = $activePageIndex;
  });
</script>

{#if $pageSelectorOpen}
  <div class="selector-overlay absolute inset-0 z-40 flex items-center justify-center bg-[#0a0b0e]/90 backdrop-blur-md">
    <div class="flex flex-col items-center gap-8">
      <span class="font-display text-sm tracking-[0.3em] uppercase text-[#6b7785]">
        Pages
      </span>
      <div class="pages-grid flex gap-4">
        {#each $profilePages as pageName, i}
          {@const page = $config?.pages.find((p) => p.name === pageName)}
          <button
            class="page-card flex flex-col items-center justify-center px-8 py-6 transition-all duration-150"
            class:active={i === highlightedIndex}
            class:current={i === $activePageIndex}
            onclick={() => goToPage(i)}
            onmouseenter={() => highlightedIndex = i}
          >
            <span class="font-display text-base text-[#e8eef2]">
              {page?.label ?? pageName}
            </span>
            <span class="mt-1 font-data text-xs text-[#6b7785]">
              {page?.zones.length ?? 0} zones
            </span>
          </button>
        {/each}
      </div>
      <span class="font-data text-xs text-[#6b7785]/60">
        Rotate to highlight · Push to select
      </span>
    </div>
  </div>
{/if}

<style>
  .page-card {
    background: #12141a;
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 6px;
    min-width: 160px;
    cursor: pointer;
  }

  .page-card:hover,
  .page-card.active {
    border-color: rgba(0, 217, 255, 0.2);
    background: rgba(0, 217, 255, 0.04);
  }

  .page-card.current {
    border-color: rgba(0, 217, 255, 0.4);
    box-shadow: 0 0 12px rgba(0, 217, 255, 0.15);
  }
</style>