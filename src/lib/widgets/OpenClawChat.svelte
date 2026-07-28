<script lang="ts">
  // OpenClaw Chat — voice + text chat interface.
  // Hold knob → record mic → Whisper STT on homelab → LLM response.
  // Ring = breathing white while listening, spinning while processing.
  // Conversation history on ultra-wide. Tap to dismiss, double-tap switches voice/text.

  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface ChatMessage {
    role: "user" | "assistant" | "system";
    content: string;
  }

  let messages = $state<ChatMessage[]>([]);
  let inputText = $state("");
  let voiceMode = $state(true);
  let isRecording = $state(false);
  let isProcessing = $state(false);
  let error = $state<string | null>(null);
  let scrollContainer: HTMLElement | null = null;

  async function loadMessages() {
    try {
      messages = await invoke<ChatMessage[]>("oc_get_messages");
    } catch {}
  }

  async function sendText() {
    if (!inputText.trim()) return;
    const text = inputText.trim();
    inputText = "";
    try {
      isProcessing = true;
      const response = await invoke<string>("oc_respond_to_text", { text });
      await loadMessages();
    } catch (e: any) {
      error = typeof e === "string" ? e : "Failed to get response";
    } finally {
      isProcessing = false;
    }
  }

  async function clearChat() {
    try {
      await invoke("oc_clear_conversation");
      await loadMessages();
    } catch {}
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendText();
    }
  }

  // Auto-scroll to bottom on new messages
  $effect(() => {
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  });

  onMount(() => {
    loadMessages();
  });
</script>

<section class="openclaw-chat flex h-full w-full flex-col">
  <!-- Header -->
  <div class="chat-header flex items-center justify-between px-6 py-2">
    <div class="flex items-center gap-2">
      <span class="label-track text-[#3a8b9e]">OpenClaw</span>
      {#if isProcessing}
        <span class="processing-dot"></span>
      {/if}
    </div>
    <div class="flex items-center gap-3">
      <button
        class="mode-btn"
        class:active={voiceMode}
        onclick={() => voiceMode = !voiceMode}
        title="Toggle voice/text mode"
      >
        {voiceMode ? "🎤" : "⌨"}
      </button>
      <button class="clear-btn" onclick={clearChat} title="Clear conversation">✕</button>
    </div>
  </div>

  <!-- Messages -->
  <div class="messages flex-1 overflow-y-auto px-6" bind:this={scrollContainer}>
    {#each messages.filter(m => m.role !== "system") as msg, i}
      <div class="msg-row flex gap-3 py-2" class:user={msg.role === "user"} class:assistant={msg.role === "assistant"}>
        <div class="msg-avatar text-xs">
          {msg.role === "user" ? "👤" : "🦁"}
        </div>
        <div class="msg-content flex-1">
          <div class="msg-role font-data text-[10px] uppercase tracking-wider"
            class:text-cyan={msg.role === "assistant"}
            class:text-teal={msg.role === "user"}
          >
            {msg.role === "user" ? "You" : "Leo"}
          </div>
          <div class="msg-text font-display text-sm text-[#e8eef2] leading-relaxed mt-0.5">
            {msg.content}
          </div>
        </div>
      </div>
    {/each}
    
    {#if isProcessing}
      <div class="msg-row flex gap-3 py-2">
        <div class="msg-avatar text-xs">🦁</div>
        <div class="msg-content flex-1">
          <div class="msg-role font-data text-[10px] uppercase tracking-wider text-cyan">Leo</div>
          <div class="typing-indicator flex items-center gap-1 mt-1">
            <span class="dot dot-1"></span>
            <span class="dot dot-2"></span>
            <span class="dot dot-3"></span>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Input -->
  <div class="input-row flex items-center gap-2 px-6 py-2 border-t border-[rgba(255,255,255,0.04)]">
    {#if voiceMode}
      <button
        class="record-btn"
        class:recording={isRecording}
        onpointerdown={() => isRecording = true}
        onpointerup={() => isRecording = false}
        onpointerleave={() => isRecording = false}
        title="Hold to speak"
      >
        {isRecording ? "●" : "🎤"}
      </button>
      <span class="hint font-data text-[10px] text-[#6b7785]">
        {isRecording ? "Listening…" : "Hold to speak"}
      </span>
    {:else}
      <input
        type="text"
        class="text-input flex-1"
        bind:value={inputText}
        onkeydown={handleKeydown}
        placeholder="Type a message…"
      />
      <button class="send-btn" onclick={sendText} disabled={!inputText.trim() || isProcessing}>
        Send
      </button>
    {/if}
  </div>

  {#if error}
    <div class="error-bar font-display text-xs text-[#f78166] px-6 py-1">{error}</div>
  {/if}
</section>

<style>
  .label-track {
    letter-spacing: 0.18em;
    text-transform: uppercase;
    font-size: 0.625rem;
    font-weight: 500;
  }

  .text-cyan { color: #00d9ff; }
  .text-teal { color: #3a8b9e; }

  .msg-row {
    border-bottom: 1px solid rgba(255, 255, 255, 0.02);
  }

  .msg-avatar {
    width: 24px;
    text-align: center;
    flex-shrink: 0;
  }

  .processing-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #00d9ff;
    animation: pulse 1.4s ease-in-out infinite;
  }

  .typing-indicator .dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: #6b7785;
  }

  .dot-1 { animation: blink 1.2s ease-in-out infinite; }
  .dot-2 { animation: blink 1.2s ease-in-out 0.2s infinite; }
  .dot-3 { animation: blink 1.2s ease-in-out 0.4s infinite; }

  @keyframes blink {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.2); }
  }

  .record-btn {
    background: rgba(0, 217, 255, 0.08);
    border: 1px solid rgba(0, 217, 255, 0.15);
    color: #00d9ff;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 150ms ease;
  }

  .record-btn.recording {
    background: rgba(0, 217, 255, 0.25);
    box-shadow: 0 0 16px rgba(0, 217, 255, 0.4);
    animation: record-pulse 1s ease-in-out infinite;
  }

  @keyframes record-pulse {
    0%, 100% { box-shadow: 0 0 8px rgba(0, 217, 255, 0.3); }
    50% { box-shadow: 0 0 20px rgba(0, 217, 255, 0.5); }
  }

  .mode-btn, .clear-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #6b7785;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .mode-btn.active, .mode-btn:hover, .clear-btn:hover {
    color: #e8eef2;
    background: rgba(255, 255, 255, 0.08);
  }

  .text-input {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #e8eef2;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 13px;
    outline: none;
  }

  .text-input:focus {
    border-color: rgba(0, 217, 255, 0.2);
  }

  .send-btn {
    background: rgba(0, 217, 255, 0.08);
    border: 1px solid rgba(0, 217, 255, 0.15);
    color: #00d9ff;
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    letter-spacing: 0.05em;
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .send-btn:not(:disabled):hover {
    background: rgba(0, 217, 255, 0.15);
  }
</style>