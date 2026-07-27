<script lang="ts">
  // Pomodoro Timer — Phase 4.1.
  // Placeholder.
  let remaining = $state(25 * 60);
  let running = $state(false);

  function toggle() { running = !running; }

  $effect(() => {
    if (!running) return;
    const id = setInterval(() => {
      if (remaining > 0) remaining--;
    }, 1000);
    return () => clearInterval(id);
  });

  const display = $derived(
    `${Math.floor(remaining / 60).toString().padStart(2, "0")}:${(remaining % 60).toString().padStart(2, "0")}`
  );
</script>

<section class="pomodoro flex h-full w-full flex-col items-center justify-center gap-3">
  <span class="label-track text-[#3a8b9e]">Focus</span>
  <span class="timer font-data text-[64px] font-300 leading-none text-white">{display}</span>
  <button class="btn" onclick={toggle}>{running ? "Pause" : "Start"}</button>
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
  .btn { background: rgba(0,217,255,0.08); border: 1px solid rgba(0,217,255,0.15); color: #00d9ff; padding: 4px 16px; border-radius: 4px; font-size: 12px; cursor: pointer; letter-spacing: 0.1em; text-transform: uppercase; }
  .btn:hover { background: rgba(0,217,255,0.15); }
</style>