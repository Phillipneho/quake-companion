<script lang="ts">
  // Notifications — flowing feed widget.
  // Colour-tagged by source. Knob scrolls, push to open/dismiss, hold to snooze.
  // Sources: Discord (blurple), GitHub (orange), Calendar (teal), System (cyan).

  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface Notification {
    id: string;
    source: "discord" | "github" | "calendar" | "system";
    title: string;
    body: string;
    timestamp: string;
    priority: "low" | "normal" | "high" | "urgent";
    url: string | null;
    read: boolean;
    snoozed: boolean;
  }

  let notifications = $state<Notification[]>([]);
  let unreadCount = $state(0);
  let selectedIndex = $state(0);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  const sourceColors: Record<string, string> = {
    discord: "#5865F2",
    github: "#f78166",
    calendar: "#3a8b9e",
    system: "#00d9ff",
  };

  const sourceLabels: Record<string, string> = {
    discord: "Discord",
    github: "GitHub",
    calendar: "Calendar",
    system: "System",
  };

  async function fetchNotifications() {
    try {
      notifications = await invoke<Notification[]>("get_unread_notifications");
      unreadCount = notifications.length;
    } catch {
      // Outside Tauri or no notifications
    }
  }

  async function dismiss(id: string) {
    try {
      await invoke("dismiss_notification", { id });
      await fetchNotifications();
    } catch {}
  }

  async function snooze(id: string) {
    try {
      await invoke("snooze_notification", { id });
      await fetchNotifications();
    } catch {}
  }

  async function openNotification(n: Notification) {
    if (n.url) {
      window.open(n.url, "_blank");
    }
    await dismiss(n.id);
  }

  function fmtTime(ts: string): string {
    const d = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "now";
    if (mins < 60) return `${mins}m`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h`;
    return d.toLocaleDateString("en-AU", { day: "numeric", month: "short" });
  }

  onMount(async () => {
    await fetchNotifications();
    pollTimer = setInterval(fetchNotifications, 30000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<section class="notifications flex h-full w-full flex-col justify-center px-6 gap-2">
  <div class="header flex items-center justify-between">
    <span class="label-track text-[#3a8b9e]">Notifications</span>
    {#if unreadCount > 0}
      <span class="badge font-data text-[10px] tabular-nums">{unreadCount} unread</span>
    {/if}
  </div>

  {#if notifications.length === 0}
    <div class="empty-state font-display text-sm text-[#6b7785] py-4">
      No new notifications
    </div>
  {:else}
    <div class="feed flex flex-col gap-1.5 overflow-hidden">
      {#each notifications.slice(0, 6) as n, i (n.id)}
        <div
          class="notif-row flex items-start gap-3 px-3 py-2 rounded transition-all"
          class:selected={i === selectedIndex}
        >
          <!-- Source colour tag -->
          <div
            class="source-tag mt-1 h-2 w-2 rounded-full shrink-0"
            style="background: {sourceColors[n.source]};"
          ></div>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="source-label font-data text-[10px] uppercase tracking-wider" style="color: {sourceColors[n.source]};">
                {sourceLabels[n.source]}
              </span>
              <span class="time font-data text-[10px] text-[#6b7785]/60">{fmtTime(n.timestamp)}</span>
            </div>
            <div class="title font-display text-sm text-[#e8eef2] truncate mt-0.5">
              {n.title}
            </div>
            {#if n.body}
              <div class="body font-display text-xs text-[#6b7785] truncate">{n.body}</div>
            {/if}
          </div>

          <!-- Actions -->
          <div class="actions flex items-center gap-1 shrink-0">
            {#if n.url}
              <button class="action-btn" onclick={() => openNotification(n)} title="Open">↗</button>
            {/if}
            <button class="action-btn" onclick={() => snooze(n.id)} title="Snooze">⏰</button>
            <button class="action-btn" onclick={() => dismiss(n.id)} title="Dismiss">✕</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .label-track {
    letter-spacing: 0.18em;
    text-transform: uppercase;
    font-size: 0.625rem;
    font-weight: 500;
  }

  .badge {
    color: #00d9ff;
    padding: 2px 6px;
    border-radius: 3px;
    background: rgba(0, 217, 255, 0.08);
  }

  .notif-row {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .notif-row:hover,
  .notif-row.selected {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.06);
  }

  .action-btn {
    background: none;
    border: none;
    color: #6b7785;
    font-size: 12px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    transition: all 120ms ease;
  }

  .action-btn:hover {
    color: #e8eef2;
    background: rgba(255, 255, 255, 0.06);
  }

  .empty-state {
    padding: 16px 0;
  }
</style>