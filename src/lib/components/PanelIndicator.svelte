<script lang="ts">
  import { activePanel, panels } from "../stores/device";
</script>

<footer
  class="indicator-bar absolute bottom-0 left-0 right-0 z-40 flex h-8 items-center justify-center gap-2"
>
  {#each panels as p, i (p.id)}
    <button
      type="button"
      class="indicator touch-target flex items-center justify-center"
      aria-label={`Go to ${p.label}`}
      aria-current={$activePanel === i}
      onclick={() => activePanel.set(i)}
    >
      <span
        class="dot transition-all duration-panel"
        class:active={$activePanel === i}
      ></span>
    </button>
  {/each}
</footer>

<style>
  .indicator-bar {
    /* faint bottom hairline */
    border-top: 1px solid rgba(0, 217, 255, 0.08);
    background: linear-gradient(to top, rgba(5, 7, 10, 0.9), rgba(5, 7, 10, 0));
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.22);
    transition-property: width, height, background-color, box-shadow;
  }
  .dot.active {
    width: 28px;
    height: 6px;
    background: var(--quake, #00d9ff);
    box-shadow: 0 0 10px rgba(0, 217, 255, 0.6);
  }
  .indicator[aria-current="true"] .dot {
    /* handled by .active */
  }
</style>