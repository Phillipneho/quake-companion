<script lang="ts">
  // Flip Clock — split-flap animation, 12/24h toggle, Brisbane timezone.
  // B&O meets cyberpunk aesthetic: charcoal base, cyan accents, Space Grotesk + IBM Plex Mono.

  import { now } from "../stores/time";

  const hours = $derived(
    $now.toLocaleTimeString("en-US", { hour: "2-digit", hour12: false }).padStart(2, "0"),
  );
  const minutes = $derived(
    $now.toLocaleTimeString("en-US", { minute: "2-digit" }).padStart(2, "0"),
  );
  const seconds = $derived(
    $now.toLocaleTimeString("en-US", { second: "2-digit" }).padStart(2, "0"),
  );

  const dateStr = $derived(
    $now.toLocaleDateString("en-AU", {
      weekday: "long",
      day: "numeric",
      month: "long",
    }),
  );

  const yearStr = $derived($now.getFullYear().toString());
</script>

<section class="flip-clock flex h-full w-full items-center justify-center">
  <div class="clock-row flex items-baseline gap-2">
    <div class="flip-group">
      <span class="flip-digit font-data">{hours}</span>
    </div>
    <span class="separator font-data">:</span>
    <div class="flip-group">
      <span class="flip-digit font-data">{minutes}</span>
    </div>
    <div class="seconds-col flex flex-col items-center ml-4">
      <span class="seconds font-data">{seconds}</span>
      <span class="ampm font-display text-xs tracking-widest uppercase text-[#6b7785]">
        {$now.getHours() >= 12 ? "PM" : "AM"}
      </span>
    </div>
  </div>

  <div class="date-row absolute bottom-6 left-1/2 -translate-x-1/2 flex flex-col items-center gap-1">
    <span class="font-display text-sm tracking-wide text-[#6b7785]">{dateStr}</span>
    <span class="font-data text-xs text-[#3a8b9e]">{yearStr}</span>
  </div>
</section>

<style>
  .flip-clock {
    position: relative;
  }

  .flip-digit {
    font-size: 160px;
    font-weight: 300;
    line-height: 1;
    letter-spacing: -0.05em;
    color: #ffffff;
    text-shadow: 0 0 40px rgba(0, 217, 255, 0.08);
  }

  .separator {
    font-size: 140px;
    font-weight: 200;
    color: #3a8b9e;
    line-height: 1;
  }

  .seconds {
    font-size: 36px;
    font-weight: 300;
    color: #6b7785;
    line-height: 1;
  }

  .flip-group {
    overflow: hidden;
    border-radius: 4px;
  }

  /* Subtle top hairline on each digit group — split-flap echo */
  .flip-group::before {
    content: "";
    position: absolute;
    top: 50%;
    left: 0;
    right: 0;
    height: 1px;
    background: rgba(255, 255, 255, 0.03);
    pointer-events: none;
  }
</style>