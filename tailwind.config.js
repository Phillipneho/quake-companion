/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        // B&O meets Cyberpunk palette
        quake: {
          DEFAULT: "#00D9FF",
          dim: "#0A6E84",
          glow: "#5EECFF",
        },
        teal: {
          DEFAULT: "#3A8B9E",
        },
        charcoal: "#0A0B0E",
        surface: "#12141A",
        elevated: "#1A1D26",
        ink: {
          950: "#0A0B0E",
          900: "#12141A",
          800: "#1A1D26",
          700: "#232733",
          600: "#2D3240",
        },
      },
      fontFamily: {
        display: ["'Space Grotesk'", "system-ui", "sans-serif"],
        mono: ["'IBM Plex Mono'", "'SF Mono'", "Menlo", "monospace"],
        sans: ["'Space Grotesk'", "system-ui", "sans-serif"],
      },
      transitionDuration: {
        panel: "180ms",
      },
      screens: {
        strip: "1900px",
      },
    },
  },
  plugins: [],
};