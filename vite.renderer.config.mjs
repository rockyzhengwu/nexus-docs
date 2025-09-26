import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    hmr: true,
  },
  resolve: {
    alias: {
      "@": path.join(__dirname, "src"),
    },
  },
});
