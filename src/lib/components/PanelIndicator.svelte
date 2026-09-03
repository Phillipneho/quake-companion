<script lang="ts">
  import { activePageIndex, profilePages, goToPage } from "../stores/layout";
  import { config } from "../stores/device";
</script>

<footer
  class="indicator-bar absolute bottom-0 left-0 right-0 z-40 flex h-8 items-center justify-center gap-3"
>
  {#each $profilePages as pageName, i (i)}
    <button
      type="button"
      class="indicator touch-target flex items-center justify-center"
      aria-label={`Go to ${$config?.pages.find((p) => p.name === pageName)?.label ?? pageName}`}
      aria-current={$activePageIndex === i}
      onclick={() => goToPage(i)}
    >
      <span class="line" class:active={$activePageIndex === i}></span>
    </button>
  {/each}
</footer>

<style>
  .indicator-bar {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    background: linear-gradient(to top, rgba(10, 11, 14, 0.85), rgba(10, 11, 14, 0));
  }
  .line {
    width: 18px;
    height: 1.5px;
    background: rgba(255, 255, 255, 0.18);
    transition: all 200ms cubic-bezier(0.22, 1, 0.36, 1);
  }
  .line.active {
    width: 36px;
    height: 2px;
    background: #00d9ff;
    box-shadow: 0 0 8px rgba(0, 217, 255, 0.4);
  }
</style>