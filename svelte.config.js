import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // Svelte 5 runs in runes mode by default; preprocess lets us use TS/PostCSS.
  preprocess: vitePreprocess(),
};