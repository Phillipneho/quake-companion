<script lang="ts">
  // Clock panel: large time, date, and weather + next-event placeholders.
  // These live in-panel; wire the placeholders to real sources (OpenClaw /
  // calendar / weather API) later.

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
  <div class="flex flex-col">
    <div class="time font-mono font-light leading-none text-white">
      {time}
    </div>
    <div class="mt-2 text-2xl font-medium text-white/70">{date}</div>
  </div>

  <div class="flex items-stretch gap-8">
    <!-- Weather placeholder -->
    <div class="card flex w-64 flex-col justify-center px-5">
      <div class="text-xs uppercase tracking-widest text-quake/70">Weather</div>
      <div class="mt-1 flex items-baseline gap-2">
        <span class="text-4xl font-light text-white">21°</span>
        <span class="text-sm text-white/50">Clear</span>
      </div>
      <div class="mt-1 text-xs text-white/40">London · feels 19°</div>
    </div>

    <!-- Next event placeholder -->
    <div class="card flex w-80 flex-col justify-center px-5">
      <div class="text-xs uppercase tracking-widest text-quake/70">Next event</div>
      <div class="mt-1 truncate text-lg font-medium text-white">
        Standup with the homelab
      </div>
      <div class="mt-0.5 text-xs text-white/50">
        Today · 14:00 → 14:30 · Google Calendar
      </div>
    </div>
  </div>
</section>

<style>
  .time {
    font-size: 180px;
    letter-spacing: -0.04em;
  }
  .card {
    border-left: 2px solid rgba(0, 217, 255, 0.4);
    background: rgba(17, 22, 31, 0.5);
    border-radius: 10px;
  }
</style>