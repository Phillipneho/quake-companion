/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        // Electric cyan — matches the QUAKE's RGB ring aesthetic.
        quake: {
          DEFAULT: "#00D9FF",
          dim: "#0A6E84",
          glow: "#5EECFF",
        },
        ink: {
          950: "#05070A",
          900: "#0A0E14",
          800: "#11161F",
          700: "#1A2230",
          600: "#27313F",
        },
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "'SF Mono'", "Menlo", "monospace"],
        sans: ["Inter", "system-ui", "sans-serif"],
      },
      transitionDuration: {
        // 150–200ms panel transitions.
        panel: "180ms",
      },
      screens: {
        // The panel is 1920x480 — extremely wide. Use min-width breakpoints so
        // the strip layout is the default.
        strip: "1900px",
      },
    },
  },
  plugins: [],
};