<script lang="ts">
  // Daily Stoic — Phase 4.4. Placeholder with a default quote.
  const quote = $state("You have power over your mind — not outside events. Realize this, and you will find strength.");
  const author = $state("Marcus Aurelius");
</script>

<section class="stoic flex h-full w-full flex-col justify-center px-8">
  <span class="label-track text-[#3a8b9e]">Stoic Daily</span>
  <blockquote class="mt-3 font-display text-base text-[#e8eef2] leading-relaxed italic">
    "{quote}"
  </blockquote>
  <span class="mt-2 font-display text-sm text-[#6b7785]">— {author}</span>
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
</style>