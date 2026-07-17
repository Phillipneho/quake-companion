// App entry — plain Vite + Svelte 5 (not SvelteKit). Mounts the App shell into
// #app. The HTML shell lives in index.html (root) / src/app.html (mirror).

import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error("#app mount target not found");
}

const app = mount(App, { target });

export default app;