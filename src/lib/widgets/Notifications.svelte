<script lang="ts">
  // Notifications — flowing feed. Phase 1.7.
  // Placeholder with styled empty state.
</script>

<section class="notifications flex h-full w-full flex-col justify-center px-6">
  <span class="label-track text-[#3a8b9e]">Notifications</span>
  <div class="mt-3 flex flex-col gap-2">
    <div class="empty-state font-display text-sm text-[#6b7785]">
      No new notifications
    </div>
  </div>
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
  .empty-state { padding: 8px 0; }
</style>