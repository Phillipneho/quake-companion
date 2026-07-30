  // Time store — single source of truth for the current time.
  // Updates every second. All widgets that need time import this.

  import { writable } from "svelte/store";

  export const now = writable<Date>(new Date());

  let intervalId: ReturnType<typeof setInterval> | undefined;

  if (typeof window !== "undefined") {
    intervalId = setInterval(() => now.set(new Date()), 1000);
  }

  // Cleanup on HMR
  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      if (intervalId) clearInterval(intervalId);
    });
  }