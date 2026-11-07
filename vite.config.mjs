import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Production: Vite builds the DOM UI (Vue) from web/. Miniquad JS plugins live in
// web/public and are copied to dist/. enzo-wasm.wasm and game assets are added by
// scripts/build-wasm.sh. The DOM layer must not contain game rules.
export default defineConfig({
  plugins: [vue()],
  root: "web",
  base: "./",
  publicDir: "public",
  build: {
    outDir: "../dist",
    emptyOutDir: false,
    sourcemap: false,
    rollupOptions: {
      output: {
        // Stable names help scripts/test-web.mjs and cache/debugging.
        entryFileNames: (chunk) => (chunk.isEntry ? "ui/main.js" : "ui/chunk-[name].js"),
        chunkFileNames: "ui/chunk-[name].js",
        assetFileNames: (assetInfo) => {
          if (assetInfo.name && assetInfo.name.endsWith(".css")) return "ui.css";
          return "ui/[name][extname]";
        },
      },
    },
  },
});
