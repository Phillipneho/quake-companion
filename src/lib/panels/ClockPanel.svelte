<script lang="ts">
  // Clock panel — the hero. Massive time, tight tracking, Plex Mono.
  // Weather for Brisbane. Next event realistic for Phillip (Program Manager at Guidant Global).

  let now = $state(new Date());

  $effect(() => {
    const id = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(id);
  });

  const time = $derived(
    now.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    }),
  );

  const date = $derived(
    now.toLocaleDateString("en-US", {
      weekday: "long",
      month: "long",
      day: "numeric",
    }),
  );
</script>

<section class="panel flex h-full w-full items-center justify-between px-16">
  <!-- Clock — the hero element -->
  <div class="flex flex-col">
    <div class="time font-data font-300 leading-none text-[#ffffff]">
      {time}
    </div>
    <div class="mt-3 font-display text-xl font-400 tracking-wide text-[#6b7785]">
      {date}
    </div>
  </div>

  <!-- Right side: embedded info cards -->
  <div class="flex items-stretch gap-6">
    <!-- Weather -->
    <div class="info-card flex w-56 flex-col justify-center px-5 py-4">
      <div class="label-track text-[#3a8b9e]">Weather</div>
      <div class="mt-2 flex items-baseline gap-2">
        <span class="font-data text-[42px] font-300 text-[#ffffff] leading-none">24°</span>
        <span class="font-display text-sm text-[#6b7785]">Clear</span>
      </div>
      <div class="mt-1.5 font-display text-xs text-[#6b7785]">
        Brisbane · feels 26°
      </div>
    </div>

    <!-- Next event -->
    <div class="info-card flex w-80 flex-col justify-center px-5 py-4">
      <div class="label-track text-[#3a8b9e]">Next event</div>
      <div class="mt-2 truncate font-display text-lg font-500 text-[#e8eef2]">
        Weekly sync — Guidant Global delivery team
      </div>
      <div class="mt-1 font-data text-xs text-[#6b7785]">
        Today · 14:00 → 14:30 · Google Calendar
      </div>
    </div>
  </div>
</section>

<style>
  .time {
    font-size: 200px;
    letter-spacing: -0.05em;
  }

  .info-card {
    background: #12141a;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.04);
  }
</style>