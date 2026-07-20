<script lang="ts">
  import { Bell, MessageSquare, AlertTriangle, CheckCircle2, type Icon as IconType } from "lucide-svelte";

  type Kind = "info" | "message" | "warning" | "success";
  interface Notif {
    kind: Kind;
    title: string;
    body: string;
    source: string;
    at: string;
  }

  const ICON: Record<Kind, IconType> = {
    info: Bell,
    message: MessageSquare,
    warning: AlertTriangle,
    success: CheckCircle2,
  };

  // Placeholder feed — wire to OpenClaw notification events later.
  const notifs: Notif[] = [
    { kind: "message", title: "Phillip", body: "Pushed the build to the homelab.", source: "Signal", at: "2m ago" },
    { kind: "warning", title: "Disk 82% full", body: "/dev/sda1 on nas — clear old snapshots.", source: "Proxmox", at: "14m ago" },
    { kind: "success", title: "Backup complete", body: "rclone → B2 finished 412 GB.", source: "n8n", at: "1h ago" },
    { kind: "info", title: "Firmware update", body: "QUAKE panel 1.2.0 available.", source: "DK-Suite", at: "3h ago" },
    { kind: "message", title: "Calendar", body: "Weekly sync in 15 minutes.", source: "Google", at: "3h ago" },
  ];

  function tint(k: Kind): string {
    switch (k) {
      case "warning":
        return "rgba(255, 180, 84, 0.06)";
      case "success":
        return "rgba(52, 230, 160, 0.05)";
      case "message":
        return "rgba(0, 217, 255, 0.05)";
      default:
        return "transparent";
    }
  }

  function iconColor(k: Kind): string {
    switch (k) {
      case "warning":
        return "#FFB454";
      case "success":
        return "#34E6A0";
      case "message":
        return "#00D9FF";
      default:
        return "#6B7785";
    }
  }
</script>

<section class="panel flex h-full w-full flex-col justify-center px-16">
  <div class="mb-3 flex items-center gap-2.5">
    <Bell size={20} class="text-[#3a8b9e]" />
    <span class="font-display text-base font-500 text-[#e8eef2]">Notifications</span>
    <span class="font-data text-xs text-[#6b7785]">· via OpenClaw</span>
  </div>

  <div class="notif-scroll flex flex-col gap-1.5 overflow-y-auto pr-2">
    {#each notifs as n, i (i)}
      {@const Icon = ICON[n.kind]}
      <div
        class="notif flex items-start gap-3 rounded-md px-4 py-2.5"
        style="background: {tint(n.kind)};"
      >
        <Icon size={18} style={`color:${iconColor(n.kind)}`} class="mt-0.5 shrink-0" />
        <div class="min-w-0 flex-1">
          <div class="flex items-baseline justify-between gap-3">
            <span class="truncate font-display text-sm font-500 text-[#e8eef2]">{n.title}</span>
            <span class="shrink-0 font-data text-xs text-[#6b7785]">{n.at} · {n.source}</span>
          </div>
          <p class="truncate font-display text-sm text-[#6b7785] mt-0.5">{n.body}</p>
        </div>
      </div>
    {/each}
  </div>
</section>

<style>
  .notif {
    border: 1px solid rgba(255, 255, 255, 0.03);
  }
  .notif-scroll {
    mask-image: linear-gradient(to bottom, transparent 0, #000 6px, #000 calc(100% - 6px), transparent 100%);
  }
</style>