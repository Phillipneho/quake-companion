<script lang="ts">
  // Pomodoro — 25/5 default, configurable. Ring = blue (focusing), green (break),
  // red flash + buzzer (time's up). Push to pause, rotate to adjust.

  import { invoke } from "@tauri-apps/api/core";

  let workMinutes = $state(25);
  let breakMinutes = $state(5);
  let remaining = $state(25 * 60);
  let running = $state(false);
  let onBreak = $state(false);
  let sessionCount = $state(0);
  let taskName = $state("");

  let intervalId: ReturnType<typeof setInterval> | undefined;

  $effect(() => {
    if (!running) {
      if (intervalId) clearInterval(intervalId);
      return;
    }
    intervalId = setInterval(() => {
      if (remaining > 0) {
        remaining--;
      } else {
        // Time's up — switch between work/break
        if (onBreak) {
          onBreak = false;
          remaining = workMinutes * 60;
          sessionCount++;
        } else {
          onBreak = true;
          remaining = breakMinutes * 60;
          // Buzzer via device
          invoke("set_led", { mode: 3 }).catch(() => {});
          setTimeout(() => invoke("set_led", { mode: onBreak ? 2 : 1 }).catch(() => {}), 500);
        }
      }
    }, 1000);
    return () => { if (intervalId) clearInterval(intervalId); };
  });

  const display = $derived(
    `${Math.floor(remaining / 60).toString().padStart(2, "0")}:${(remaining % 60).toString().padStart(2, "0")}`
  );

  const phase = $derived(onBreak ? "Break" : "Focus");
  const phaseColor = $derived(onBreak ? "#3a8b9e" : "#00d9ff");
  const progressPct = $derived(onBreak
    ? ((breakMinutes * 60 - remaining) / (breakMinutes * 60)) * 100
    : ((workMinutes * 60 - remaining) / (workMinutes * 60)) * 100
  );

  function toggle() { running = !running; }

  function reset() {
    running = false;
    onBreak = false;
    remaining = workMinutes * 60;
  }

  function adjust(delta: number) {
    if (onBreak) {
      breakMinutes = Math.max(1, Math.min(60, breakMinutes + delta));
      if (!running) remaining = breakMinutes * 60;
    } else {
      workMinutes = Math.max(1, Math.max(1, workMinutes + delta));
      if (!running) remaining = workMinutes * 60;
    }
  }
</script>

<section class="pomodoro flex h-full w-full flex-col items-center justify-center gap-2">
  <div class="phase-label font-display text-xs tracking-[0.3em] uppercase" style="color: {phaseColor}">
    {phase}
  </div>
  <div class="timer font-data text-[72px] font-300 leading-none tabular-nums" style="color: {onBreak ? '#3a8b9e' : '#ffffff'}">
    {display}
  </div>

  <!-- Progress ring -->
  <div class="progress-track w-48 h-0.5 bg-[rgba(255,255,255,0.06)] rounded-full mt-1">
    <div class="progress-fill h-full rounded-full transition-all duration-1000"
      style="width: {progressPct}%; background: {phaseColor};"
    ></div>
  </div>

  {#if taskName}
    <div class="task-name font-display text-sm text-[#6b7785] truncate max-w-full px-4">{taskName}</div>
  {/if}

  <div class="controls flex items-center gap-3 mt-2">
    <button class="ctrl-btn" onclick={() => adjust(-5)}>−5</button>
    <button class="ctrl-btn primary" onclick={toggle}>{running ? "Pause" : "Start"}</button>
    <button class="ctrl-btn" onclick={() => adjust(5)}>+5</button>
    <button class="ctrl-btn" onclick={reset} title="Reset">↻</button>
  </div>

  <div class="session-count font-data text-[10px] text-[#6b7785]/60 mt-1">
    {sessionCount} sessions completed
  </div>
</section>

<style>
  .ctrl-btn {
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.06);
    color: #e8eef2;
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    transition: all 120ms ease;
  }
  .ctrl-btn:hover { background: rgba(255,255,255,0.08); }
  .ctrl-btn.primary {
    background: rgba(0,217,255,0.08);
    border-color: rgba(0,217,255,0.15);
    color: #00d9ff;
    padding: 4px 16px;
  }
  .ctrl-btn.primary:hover { background: rgba(0,217,255,0.15); }
</style>