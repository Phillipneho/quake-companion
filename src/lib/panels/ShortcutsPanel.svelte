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

  // Icon registry — keys referenced from shortcut JSON.
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

  // Config is JSON in localStorage under quake.shortcuts; fall back to defaults.
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
        // Placeholder: a real shell-command bridge would be a Tauri command.
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
  <div class="grid grid-cols-8 gap-4">
    {#each shortcuts as s, i (i)}
      {@const Comp = ICONS[s.icon] ?? Globe}
      <button
        class="shortcut touch-target flex flex-col items-center justify-center gap-2 rounded-xl bg-ink-800/60 px-2 py-3 transition-colors hover:bg-ink-700/80"
        onclick={() => run(s)}
      >
        <Comp size={32} class="text-quake" />
        <span class="text-sm font-medium text-white/85">{s.label}</span>
      </button>
    {/each}
  </div>

  {#if toast}
    <div class="pointer-events-none absolute bottom-12 left-1/2 -translate-x-1/2 rounded-full bg-quake/15 px-4 py-1 text-sm text-quake-glow">
      {toast}
    </div>
  {/if}
</section>

<style>
  .shortcut {
    border: 1px solid rgba(0, 217, 255, 0.1);
  min-height: 120px;
  }
  .shortcut:hover {
    border-color: rgba(0, 217, 255, 0.4);
    box-shadow: 0 0 14px rgba(0, 217, 255, 0.12);
  }
</style>