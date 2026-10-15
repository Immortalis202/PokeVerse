import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import catalogPreview from "./scripts/catalog-preview.mjs";
export default defineConfig({
  plugins: [react(), catalogPreview()],
  clearScreen: false,
  server: { strictPort: true, watch: { ignored: ["**/src-tauri/**"] } },
  build: { target: "es2022" },
});
