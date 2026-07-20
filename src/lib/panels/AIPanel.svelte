<script lang="ts">
  import { Send, Bot, User, Settings2, ChevronRight } from "lucide-svelte";

  interface ChatMsg {
    role: "user" | "assistant" | "system";
    content: string;
  }

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
          content: `⚠️ Could not reach ${endpoint} — ${e instanceof Error ? e.message : "error"}. Tap the gear icon to configure.`,
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
    <!-- Header -->
    <div class="flex items-center justify-between pb-2">
      <div class="flex items-center gap-2.5">
        <Bot size={20} class="text-[#3a8b9e]" />
        <span class="font-display text-base font-500 text-[#e8eef2]">Quick chat</span>
        <span class="font-data text-xs text-[#6b7785]">· {model}</span>
      </div>
      <button
        class="touch-target flex items-center gap-1.5 rounded px-2 py-1 transition-colors hover:text-[#00d9ff]"
        class:text-qua-since={!showSettings}
        onclick={() => (showSettings = !showSettings)}
      >
        <Settings2 size={16} class="text-[#6b7785] transition-colors" />
        <span class="font-display text-xs text-[#6b7785]">Settings</span>
      </button>
    </div>

    <!-- Settings drawer -->
    {#if showSettings}
      <div class="settings-drawer mb-2 flex flex-col gap-2 px-4 py-3">
        <label class="flex items-center gap-3">
          <span class="label-track w-20 text-[#6b7785]">Endpoint</span>
          <input
            class="flex-1 rounded bg-[#0a0b0e] px-3 py-1.5 font-data text-xs text-[#e8eef2] outline-none ring-1 ring-[rgba(0,217,255,0.08)] focus:ring-[rgba(0,217,255,0.3)]"
            bind:value={endpoint}
          />
        </label>
        <label class="flex items-center gap-3">
          <span class="label-track w-20 text-[#6b7785]">Model</span>
          <input
            class="flex-1 rounded bg-[#0a0b0e] px-3 py-1.5 font-data text-xs text-[#e8eef2] outline-none ring-1 ring-[rgba(0,217,255,0.08)] focus:ring-[rgba(0,217,255,0.3)]"
            bind:value={model}
          />
        </label>
      </div>
    {/if}

    <!-- Messages -->
    <div bind:this={scrollEl} class="msg-scroll flex-1 overflow-y-auto pr-2">
      {#each messages as m, i (i)}
        <div class="msg flex gap-3 py-1.5" class:user={m.role === "user"}>
          {#if m.role === "user"}
            <div class="msg-avatar user-avatar flex h-7 w-7 shrink-0 items-center justify-center rounded">
              <User size={14} class="text-[#00d9ff]" />
            </div>
          {:else}
            <div class="msg-avatar assistant-avatar flex h-7 w-7 shrink-0 items-center justify-center rounded">
              <Bot size={14} class="text-[#6b7785]" />
            </div>
          {/if}
          <p class="text-sm leading-relaxed text-[#e8eef2]/90 pt-0.5">{m.content}</p>
        </div>
      {/each}
    </div>

    <!-- Input -->
    <div class="mt-2 flex items-center gap-3 pt-2.5" style="border-top: 1px solid rgba(255,255,255,0.04);">
      <input
        class="flex-1 rounded-lg bg-[#12141a] px-4 py-2.5 font-display text-sm text-[#e8eef2] outline-none ring-1 ring-[rgba(255,255,255,0.04)] focus:ring-[rgba(0,217,255,0.2)]"
        placeholder="Ask the panel…"
        bind:value={input}
        onkeydown={onKey}
      />
      <button
        class="touch-target flex items-center gap-2 rounded-lg px-5 py-2.5 font-display text-sm font-500 transition-opacity disabled:opacity-30"
        style="background: #00d9ff; color: #0a0b0e;"
        disabled={busy || !input.trim()}
        onclick={() => void send()}
      >
        <Send size={16} />
        Send
      </button>
    </div>
  </div>
</section>

<style>
  .user-avatar {
    background: rgba(0, 217, 255, 0.06);
  }
  .assistant-avatar {
    background: rgba(255, 255, 255, 0.03);
  }
  .msg.user p {
    color: #e8eef2;
  }
  .msg-scroll {
    mask-image: linear-gradient(to bottom, transparent 0, #000 6px, #000 calc(100% - 6px), transparent 100%);
  }
  .settings-drawer {
    background: #12141a;
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 6px;
  }
</style>