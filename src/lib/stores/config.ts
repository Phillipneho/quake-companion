// Config store — separated from device.ts to break circular dependency.
// layout.ts and device.ts both need config, and device.ts imports from layout.ts,
// so config lives here to avoid the cycle.

import { writable } from "svelte/store";
import type { Config } from "./device";

export const config = writable<Config | null>(null);