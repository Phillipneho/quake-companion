<script lang="ts">
  import { Send, Bot, User, Settings2 } from "lucide-svelte";

  interface ChatMsg {
    role: "user" | "assistant" | "system";
    content: string;
  }

  // Configurable OpenAI-compatible endpoint. Defaults to a local Ollama
  // (OpenClaw exposes the same /v1/chat/completions shape). Editable in-panel.
  const DEFAULT_ENDPOINT = "http://localhost:11434/v1/chat/completions";
  const DEFAULT_MODEL = "glm-4.6";

  let endpoint = $state(localStorage.getItem("quake.ai.endpoint") || DEFAULT_ENDPOINT);
  let model = $state(localStorage.getItem("quake.ai.model") || DEFAULT_MODEL);
  let showSettings = $state(false);

  let messages = $state<ChatMsg[]>([
    {
      role: "assistant",
      content: "Ask me anything — I run against your local OpenAI-compatible endpoint.",
    },
  ]);
  let input = $state("");
  let busy = $state(false);
  let scrollEl: HTMLDivElement | null = $state(null);

  $effect(() => {
    localStorage.setItem("quake.ai.endpoint", endpoint);
    localStorage.setItem("quake.ai.model", model);
  });

  // Auto-scroll to newest message.
  $effect(() => {
    messages;
    if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
  });

  async function send() {
    const text = input.trim();
    if (!text || busy) return;
    input = "";
    messages = [...messages, { role: "user", content: text }];
    busy = true;
    try {
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model,
          messages: [...messages.filter((m) => m.role !== "system"), { role: "user", content: text }],
          stream: false,
        }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      const reply: string =
        data?.choices?.[0]?.message?.content ??
        data?.message?.content ??
        "(no reply)";
      messages = [...messages, { role: "assistant", content: reply }];
    } catch (e) {
      messages = [
        ...messages,
        {
          role: "assistant",
          content: `⚠️ Could not reach ${endpoint} — ${e instanceof Error ? e.message : "error"}. Tap ⚙ to set your endpoint.`,
        },
      ];
    } finally {
      busy = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void send();
    }
  }
</script>

<section class="panel flex h-full w-full items-stretch px-16 py-3">
  <div class="flex h-full w-full flex-col">
    <div class="flex items-center justify-between pb-2">
      <div class="flex items-center gap-2 text-quake">
        <Bot size={22} />
        <span class="text-lg font-medium text-white">Quick chat</span>
        <span class="text-xs text-white/40">· {model}</span>
      </div>
      <button
        class="touch-target flex items-center gap-1 rounded-md px-2 py-1 text-xs text-white/60 hover:text-quake"
        onclick={() => (showSettings = !showSettings)}
      >
        <Settings2 size={16} /> Settings
      </button>
    </div>

    {#if showSettings}
      <div class="mb-2 flex flex-col gap-1 rounded-md bg-ink-800/60 p-2 text-xs">
        <label class="flex items-center gap-2">
          <span class="w-20 text-white/50">Endpoint</span>
          <input
            class="flex-1 rounded bg-ink-900 px-2 py-1 text-white outline-none ring-1 ring-quake/20 focus:ring-quake/60"
            bind:value={endpoint}
          />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-20 text-white/50">Model</span>
          <input
            class="flex-1 rounded bg-ink-900 px-2 py-1 text-white outline-none ring-1 ring-quake/20 focus:ring-quake/60"
            bind:value={model}
          />
        </label>
      </div>
    {/if}

    <div bind:this={scrollEl} class="msg-scroll flex-1 overflow-y-auto pr-2">
      {#each messages as m, i (i)}
        <div class="msg flex gap-3 py-1.5" class:user={m.role === "user"}>
          {#if m.role === "user"}
            <User size={18} class="mt-1 shrink-0 text-quake" />
          {:else}
            <Bot size={18} class="mt-1 shrink-0 text-white/40" />
          {/if}
          <p class="text-base leading-snug text-white/90">{m.content}</p>
        </div>
      {/each}
    </div>

    <div class="mt-2 flex items-center gap-2 border-t border-quake/15 pt-2">
      <input
        class="flex-1 rounded-lg bg-ink-800 px-4 py-2.5 text-base text-white outline-none ring-1 ring-quake/20 focus:ring-quake/60"
        placeholder="Ask the panel…"
        bind:value={input}
        onkeydown={onKey}
      />
      <button
        class="touch-target flex items-center gap-2 rounded-lg bg-quake px-5 py-2.5 font-medium text-ink-950 disabled:opacity-40"
        disabled={busy || !input.trim()}
        onclick={() => void send()}
      >
        <Send size={18} />
        Send
      </button>
    </div>
  </div>
</section>

<style>
  .msg.user p {
    color: #d9f6ff;
  }
  .msg-scroll {
    mask-image: linear-gradient(to bottom, transparent 0, #000 8px, #000 calc(100% - 8px), transparent 100%);
  }
</style>