<script lang="ts">
  // Calendar — next meeting, day timeline, knob scrolls days.
  // Push to join (Zoom/Teams link detected). Colour-coded by calendar.
  // Placeholder using static data until Google Calendar API is wired up.

  import { now } from "../stores/time";

  interface Event {
    title: string;
    time: string;
    duration_min: number;
    source: "google" | "outlook" | "manual";
    join_url?: string;
    color: string;
  }

  // Placeholder events — will be replaced with Google Calendar API data
  let events = $state<Event[]>([
    { title: "Weekly sync — Guidant delivery", time: "14:00", duration_min: 30, source: "google", join_url: "https://meet.google.com/abc-defg-hij", color: "#3a8b9e" },
    { title: "BHP MSP review", time: "15:30", duration_min: 60, source: "google", join_url: "https://zoom.us/j/123456789", color: "#00d9ff" },
    { title: "1:1 with Sarah", time: "16:45", duration_min: 30, source: "outlook", color: "#f78166" },
  ]);

  let selectedDay = $state(0);

  const dayLabel = $derived(
    new Date(Date.now() + selectedDay * 86400000).toLocaleDateString("en-AU", { weekday: "short", day: "numeric", month: "short" })
  );

  const timeStr = $derived($now.toLocaleTimeString("en-AU", { hour: "2-digit", minute: "2-digit", hour12: false }));

  function joinMeeting(url: string) {
    window.open(url, "_blank");
  }

  function fmtDuration(min: number): string {
    if (min < 60) return `${min}m`;
    const h = Math.floor(min / 60);
    const m = min % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
  }
</script>

<section class="calendar flex h-full w-full flex-col justify-center px-6 gap-2">
  <div class="header flex items-center justify-between">
    <span class="label-track text-[#3a8b9e]">Calendar</span>
    <span class="font-data text-[10px] text-[#6b7785]">{dayLabel} · {timeStr}</span>
  </div>

  {#if events.length === 0}
    <div class="empty font-display text-sm text-[#6b7785] py-4">No upcoming events</div>
  {:else}
    <div class="event-list flex flex-col gap-1.5">
      {#each events.slice(0, 4) as ev}
        <div class="event-row flex items-start gap-3 px-3 py-2 rounded" style="border-left: 2px solid {ev.color};">
          <div class="time-col flex flex-col items-center shrink-0">
            <span class="font-data text-sm text-[#e8eef2] tabular-nums">{ev.time}</span>
            <span class="font-data text-[10px] text-[#6b7785]">{fmtDuration(ev.duration_min)}</span>
          </div>
          <div class="flex-1 min-w-0">
            <div class="title font-display text-sm text-[#e8eef2] truncate">{ev.title}</div>
            <div class="source font-data text-[10px] text-[#6b7785] uppercase tracking-wider mt-0.5">{ev.source}</div>
          </div>
          {#if ev.join_url}
            <button class="join-btn" onclick={() => joinMeeting(ev.join_url)} title="Join meeting">
              Join
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
  .event-row { background: rgba(255,255,255,0.02); }
  .join-btn {
    background: rgba(0,217,255,0.08);
    border: 1px solid rgba(0,217,255,0.15);
    color: #00d9ff;
    padding: 3px 10px;
    border-radius: 3px;
    font-size: 11px;
    cursor: pointer;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }
  .join-btn:hover { background: rgba(0,217,255,0.15); }
</style>