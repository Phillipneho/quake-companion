<script lang="ts">
  // Weather widget — Brisbane weather via open-meteo API (free, no key).
  // Current temp, conditions, high/low. 3-hour forecast strip.

  import { onMount } from "svelte";
  import { now } from "../stores/time";

  interface WeatherData {
    current: { temp: number; condition: string; code: number };
    today: { high: number; low: number };
    forecast: { time: string; temp: number; code: number }[];
  }

  let weather = $state<WeatherData | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Weather code mapping (WMO codes → text)
  function codeToText(code: number): string {
    const map: Record<number, string> = {
      0: "Clear", 1: "Mostly clear", 2: "Partly cloudy", 3: "Overcast",
      45: "Fog", 48: "Fog", 51: "Drizzle", 53: "Drizzle", 55: "Drizzle",
      61: "Rain", 63: "Rain", 65: "Heavy rain", 71: "Snow", 73: "Snow",
      75: "Heavy snow", 80: "Showers", 81: "Showers", 82: "Heavy showers",
      95: "Storm", 96: "Storm", 99: "Severe storm",
    };
    return map[code] ?? "—";
  }

  // Brisbane coords
  const LAT = -27.47;
  const LON = 153.02;

  async function fetchWeather() {
    try {
      const url = `https://api.open-meteo.com/v1/forecast?latitude=${LAT}&longitude=${LON}&current=temperature_2m,weather_code&daily=temperature_2m_max,temperature_2m_min&hourly=temperature_2m,weather_code&forecast_hours=12&timezone=Australia%2FBrisbane`;
      const res = await fetch(url);
      const data = await res.json();

      const forecast: { time: string; temp: number; code: number }[] = [];
      const nowHour = new Date().getHours();
      for (let i = 0; i < 6; i++) {
        const idx = i;
        if (data.hourly && data.hourly.time[idx]) {
          forecast.push({
            time: data.hourly.time[idx],
            temp: Math.round(data.hourly.temperature_2m[idx]),
            code: data.hourly.weather_code[idx],
          });
        }
      }

      weather = {
        current: {
          temp: Math.round(data.current.temperature_2m),
          condition: codeToText(data.current.weather_code),
          code: data.current.weather_code,
        },
        today: {
          high: Math.round(data.daily.temperature_2m_max[0]),
          low: Math.round(data.daily.temperature_2m_min[0]),
        },
        forecast,
      };
      error = null;
    } catch (e) {
      error = "Failed to load weather";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchWeather();
    // Refresh every 10 minutes
    const id = setInterval(fetchWeather, 600000);
    return () => clearInterval(id);
  });

  const timeStr = $derived(
    $now.toLocaleTimeString("en-AU", { hour: "2-digit", minute: "2-digit", hour12: false }),
  );
</script>

<section class="weather-widget flex h-full w-full flex-col justify-center px-8">
  {#if loading}
    <div class="flex items-center gap-3 text-[#6b7785]">
      <span class="font-display text-sm tracking-widest uppercase">Loading…</span>
    </div>
  {:else if error}
    <div class="font-display text-sm text-[#6b7785]">{error}</div>
  {:else if weather}
    <!-- Current -->
    <div class="current-row flex items-baseline gap-4">
      <span class="temp font-data text-[64px] font-300 leading-none text-white">
        {weather.current.temp}°
      </span>
      <div class="flex flex-col gap-0.5">
        <span class="condition font-display text-base text-[#e8eef2]">
          {weather.current.condition}
        </span>
        <span class="hilo font-data text-xs text-[#6b7785]">
          H {weather.today.high}° · L {weather.today.low}°
        </span>
      </div>
    </div>

    <!-- Location + time -->
    <div class="meta-row mt-2 flex items-center gap-2">
      <span class="font-display text-xs tracking-widest uppercase text-[#3a8b9e]">Brisbane</span>
      <span class="text-[#6b7785]/40">·</span>
      <span class="font-data text-xs text-[#6b7785]">{timeStr}</span>
    </div>

    <!-- 3-hour forecast strip -->
    <div class="forecast-strip mt-4 flex gap-3">
      {#each weather.forecast.slice(0, 4) as f}
        <div class="forecast-cell flex flex-col items-center gap-1 px-2 py-1.5">
          <span class="font-data text-[10px] text-[#6b7785]">
            {new Date(f.time).toLocaleTimeString("en-AU", { hour: "2-digit", hour12: false })}h
          </span>
          <span class="font-data text-sm text-[#e8eef2]">{f.temp}°</span>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .weather-widget {
    background: transparent;
  }

  .forecast-cell {
    background: rgba(255, 255, 255, 0.03);
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.04);
    min-width: 48px;
  }
</style>