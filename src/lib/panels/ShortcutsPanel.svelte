<script lang="ts">
  import {
    Github,
    Globe,
    Terminal,
    Webhook,
    Music,
    Calendar,
    Mail,
    Server,
    Settings,
    type Icon as IconType,
  } from "lucide-svelte";

  type Action =
    | { kind: "url"; href: string }
    | { kind: "command"; command: string }
    | { kind: "webhook"; url: string };

  interface Shortcut {
    icon: keyof typeof ICONS;
    label: string;
    action: Action;
  }

  const ICONS: Record<string, IconType> = {
    Github,
    Globe,
    Terminal,
    Webhook,
    Music,
    Calendar,
    Mail,
    Server,
    Settings,
  };

  const DEFAULT_SHORTCUTS: Shortcut[] = [
    { icon: "Globe", label: "Homelab", action: { kind: "url", href: "https://homelab.local" } },
    { icon: "Github", label: "GitHub", action: { kind: "url", href: "https://github.com" } },
    { icon: "Server", label: "Proxmox", action: { kind: "url", href: "https://proxmox.local:8006" } },
    { icon: "Terminal", label: "SSH box", action: { kind: "command", command: "ssh homelab" } },
    { icon: "Music", label: "Spotify", action: { kind: "url", href: "https://open.spotify.com" } },
    { icon: "Calendar", label: "Calendar", action: { kind: "url", href: "https://calendar.google.com" } },
    { icon: "Mail", label: "Mail", action: { kind: "url", href: "https://mail.google.com" } },
    { icon: "Webhook", label: "Deploy", action: { kind: "webhook", url: "http://localhost:5678/webhook/deploy" } },
  ];

  let shortcuts = $state<Shortcut[]>(DEFAULT_SHORTCUTS);
  let toast = $state<string | null>(null);

  $effect(() => {
    const raw = localStorage.getItem("quake.shortcuts");
    if (raw) {
      try {
        const parsed = JSON.parse(raw) as Shortcut[];
        if (Array.isArray(parsed) && parsed.length) shortcuts = parsed;
      } catch {
        /* keep defaults */
      }
    }
  });

  function run(s: Shortcut) {
    switch (s.action.kind) {
      case "url":
        window.open(s.action.href, "_blank");
        toast = `Opening ${s.action.href}`;
        break;
      case "command":
        toast = `Run: ${s.action.command}`;
        break;
      case "webhook":
        void fetch(s.action.url, { method: "POST" })
          .then(() => (toast = `Webhook fired: ${s.label}`))
          .catch(() => (toast = `Webhook failed: ${s.label}`));
        break;
    }
    setTimeout(() => (toast = null), 2000);
  }
</script>

<section class="panel flex h-full w-full flex-col justify-center px-16">
  <div class="grid grid-cols-8 gap-3">
    {#each shortcuts as s, i (i)}
      {@const Comp = ICONS[s.icon] ?? Globe}
      <button
        class="shortcut touch-target flex flex-col items-center justify-center gap-3 rounded-lg px-2 py-4 transition-all"
        onclick={() => run(s)}
      >
        <Comp size={28} class="text-[#3a8b9e] transition-colors" />
        <span class="font-display text-xs font-500 text-[#e8eef2]/80">{s.label}</span>
      </button>
    {/each}
  </div>

  {#if toast}
    <div class="pointer-events-none absolute bottom-12 left-1/2 -translate-x-1/2 rounded-full px-4 py-1.5 font-display text-xs text-[#00d9ff]" style="background: rgba(0,217,255,0.08);">
      {toast}
    </div>
  {/if}
</section>

<style>
  .shortcut {
    background: #12141a;
    border: 1px solid rgba(255, 255, 255, 0.03);
    min-height: 110px;
  }
  .shortcut:hover {
    background: #1a1d26;
    border-color: rgba(0, 217, 255, 0.15);
  }
  .shortcut:hover :global(svg) {
    color: #00d9ff !important;
  }
</style>