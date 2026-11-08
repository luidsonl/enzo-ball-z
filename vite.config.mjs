import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Production: Vite builds the DOM UI (Vue) from web/. Miniquad JS plugins live in
// web/public and are copied to dist/. enzo-wasm.wasm and game assets are added by
// scripts/build-wasm.sh. The DOM layer must not contain game rules.
export default defineConfig({
  plugins: [
    vue(),
    {
      name: "ignore-miniquad-classic-scripts",
      enforce: "pre",
      transformIndexHtml(html) {
        // Keep miniquad/quad-storage classic globals out of Vite/Rollup HTML asset bundling.
        return html.replace(
          /<script(\s+)src="\.\/(mq_js_bundle\.js|sapp_jsutils\.js|quad-storage\.js)">/g,
          '<script$1src="/$2" data-vite-ignore>'
        );
      },
    },
  ],
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
