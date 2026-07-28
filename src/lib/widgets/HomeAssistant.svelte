<script lang="ts">
  // Home Assistant — smart home control widget.
  // Shows entity states grouped by domain. Knob adjusts brightness/volume,
  // push toggles. Supports lights, switches, media players, climate, sensors.

  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface HaEntitySummary {
    entity_id: string;
    friendly_name: string;
    state: string;
    domain: string;
  }

  let entities = $state<HaEntitySummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  // Group entities by domain
  const grouped = $derived.by(() => {
    const groups: Record<string, HaEntitySummary[]> = {};
    for (const e of entities) {
      if (!groups[e.domain]) groups[e.domain] = [];
      groups[e.domain].push(e);
    }
    return groups;
  });

  const domainLabels: Record<string, string> = {
    light: "Lights",
    switch: "Switches",
    media_player: "Media",
    climate: "Climate",
    sensor: "Sensors",
    person: "People",
    zone: "Zones",
  };

  const domainIcons: Record<string, string> = {
    light: "💡",
    switch: "🔌",
    media_player: "🔊",
    climate: "🌡",
    sensor: "📊",
    person: "👤",
    zone: "📍",
  };

  async function fetchStates() {
    try {
      entities = await invoke<HaEntitySummary[]>("ha_get_states");
      error = null;
    } catch (e: any) {
      error = typeof e === "string" ? e : "Failed to fetch HA states";
    } finally {
      loading = false;
    }
  }

  async function toggle(entity_id: string) {
    try {
      await invoke("ha_toggle", { entityId: entity_id });
      await fetchStates();
    } catch (e: any) {
      error = typeof e === "string" ? e : "Toggle failed";
    }
  }

  async function turnOn(entity_id: string) {
    try {
      await invoke("ha_turn_on", { entityId: entity_id });
      await fetchStates();
    } catch {}
  }

  async function turnOff(entity_id: string) {
    try {
      await invoke("ha_turn_off", { entityId: entity_id });
      await fetchStates();
    } catch {}
  }

  function isOn(state: string): boolean {
    return state === "on" || state === "playing" || state === "home";
  }

  onMount(async () => {
    await fetchStates();
    pollTimer = setInterval(fetchStates, 10000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<section class="ha-widget flex h-full w-full flex-col justify-center px-6 gap-2">
  <div class="header flex items-center justify-between">
    <span class="label-track text-[#3a8b9e]">Home</span>
    {#if entities.length > 0}
      <span class="font-data text-[10px] text-[#6b7785]">{entities.length} entities</span>
    {/if}
  </div>

  {#if loading}
    <div class="font-display text-sm text-[#6b7785] py-4">Loading…</div>
  {:else if error}
    <div class="font-display text-sm text-[#6b7785] py-2">{error}</div>
  {:else}
    <div class="entity-list flex flex-col gap-1 overflow-hidden">
      {#each Object.entries(grouped).sort(([a], [b]) => a.localeCompare(b)) as [domain, ents]}
        <div class="domain-group">
          <div class="domain-header flex items-center gap-1.5 py-1">
            <span class="domain-icon text-xs">{domainIcons[domain] ?? "📦"}</span>
            <span class="label-track text-[#6b7785]">{domainLabels[domain] ?? domain}</span>
          </div>
          {#each ents.slice(0, 4) as ent}
            <div class="entity-row flex items-center justify-between px-2 py-1 rounded" class:on={isOn(ent.state)}>
              <span class="entity-name font-display text-xs text-[#e8eef2] truncate flex-1">
                {ent.friendly_name}
              </span>
              <button
                class="toggle-btn"
                class:on={isOn(ent.state)}
                onclick={() => toggle(ent.entity_id)}
                title={ent.state}
              >
                {isOn(ent.state) ? "●" : "○"}
              </button>
            </div>
          {/each}
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

  .domain-group {
    margin-bottom: 4px;
  }

  .entity-row {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .entity-row.on {
    background: rgba(0, 217, 255, 0.04);
    border-color: rgba(0, 217, 255, 0.08);
  }

  .toggle-btn {
    background: none;
    border: none;
    font-size: 10px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    color: #6b7785;
    transition: all 120ms ease;
  }

  .toggle-btn.on {
    color: #00d9ff;
  }

  .toggle-btn:hover {
    color: #e8eef2;
    background: rgba(255, 255, 255, 0.06);
  }
</style>