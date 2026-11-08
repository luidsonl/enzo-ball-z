import { defineConfig, createLogger } from "vite";
import vue from "@vitejs/plugin-vue";

const logger = createLogger();
const originalWarn = logger.warn.bind(logger);
const originalWarnOnce = logger.warnOnce.bind(logger);
function isMiniquadScriptWarning(msg) {
  const text = String(msg || "");
  return (
    text.includes("can't be bundled without type=\"module\" attribute") &&
    (text.includes("mq_js_bundle.js") ||
      text.includes("sapp_jsutils.js") ||
      text.includes("quad-storage.js"))
  );
}
logger.warn = (msg, options) => {
  if (isMiniquadScriptWarning(msg)) return;
  originalWarn(msg, options);
};
logger.warnOnce = (msg, options) => {
  if (isMiniquadScriptWarning(msg)) return;
  originalWarnOnce(msg, options);
};

// Production: Vite builds the DOM UI (Vue) from web/. Miniquad JS plugins live in
// web/public and are copied to dist/. enzo-wasm.wasm and game assets are added by
// scripts/build-wasm.sh. The DOM layer must not contain game rules.
export default defineConfig({
  customLogger: logger,
  plugins: [
    vue(),
    {
      name: "miniquad-script-paths",
      enforce: "post",
      transformIndexHtml(html) {
        // Keep classic miniquad/quad-storage globals out of bundling and use relative
        // paths so the same build works locally and on GitHub Pages subpaths (CI/CD).
        return html.replace(
          /<script(\s+)src="(?:\/+|\.\/)(mq_js_bundle\.js|sapp_jsutils\.js|quad-storage\.js)"([^>]*)>/g,
          (_m, ws, file, rest) => {
            const attrs = rest.includes("data-vite-ignore") ? rest : `${rest} data-vite-ignore`;
            return `<script${ws}src="./${file}"${attrs}>`;
          }
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
