#!/usr/bin/env node
// Smoke test for the web/WASM build output.
// Usage: node scripts/smoke-web.mjs [--root dist] [--serve]

import { access, readFile, stat } from "node:fs/promises";
import { constants } from "node:fs";
import { createServer } from "node:http";
import { extname, join, resolve } from "node:path";

const args = process.argv.slice(2);
const root = resolve(args.includes("--root") ? args[args.indexOf("--root") + 1] : "dist");
const serve = args.includes("--serve");

const required = [
  "index.html",
  "enzo-wasm.wasm",
  "mq_js_bundle.js",
  "sapp_jsutils.js",
  "quad-storage.js",
  "ui.css",
  "ui/main.js",
  "ui/bridge.js",
  "ui/mobile-controls.js",
  "ui/state.js",
  "ui/hud.js",
  "ui/menus.js",
  "assets",
];

const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".png": "image/png",
};

let failed = 0;
function ok(msg) { console.log(`ok: ${msg}`); }
function fail(msg) { console.error(`fail: ${msg}`); failed++; }

for (const rel of required) {
  const file = join(root, rel);
  try {
    await access(file, constants.R_OK);
    const info = await stat(file);
    ok(`${rel}${info.isFile() ? ` (${info.size} bytes)` : "/"}`);
  } catch {
    fail(`missing ${rel}`);
  }
}

try {
  const qs = await readFile(join(root, "quad-storage.js"), "utf8");
  if (/name:\s*["']quad_storage["']/.test(qs)) ok("quad-storage plugin name present");
  else fail("quad-storage plugin name missing");
  if (/version\s*:\s*(\(|0x|\d)/.test(qs) && !/version:\s*["']0\.1\.4["']/.test(qs)) {
    ok("quad-storage version is not stale 0.1.4 string");
  } else {
    fail("quad-storage version looks stale/incompatible");
  }
} catch (e) {
  fail(`quad-storage.js check: ${e.message}`);
}

try {
  const html = await readFile(join(root, "index.html"), "utf8");
  if (html.includes('type="module"') && html.includes("ui/main.js")) ok("index.html loads modular UI");
  else fail("index.html missing modular UI entry");
  if (html.includes("quad-storage.js")) ok("index.html loads quad-storage plugin");
  else fail("index.html missing quad-storage plugin");
} catch (e) {
  fail(`index.html check: ${e.message}`);
}

if (serve) {
  await new Promise((resolvePromise) => {
    const server = createServer(async (req, res) => {
      try {
        const url = new URL(req.url ?? "/", "http://localhost");
        const requested = url.pathname.endsWith("/") ? `${url.pathname}index.html` : url.pathname;
        const file = join(root, requested);
        if (!file.startsWith(root)) return res.writeHead(403).end();
        const body = await readFile(file);
        res.writeHead(200, { "Content-Type": mime[extname(file)] ?? "application/octet-stream" });
        res.end(body);
      } catch {
        res.writeHead(404).end();
      }
    });
    server.listen(0, "127.0.0.1", async () => {
      const { port } = server.address();
      ok(`smoke server on http://127.0.0.1:${port}`);
      for (const path of ["/", "/index.html", "/enzo-wasm.wasm", "/ui/main.js", "/quad-storage.js"]) {
        const res = await fetch(`http://127.0.0.1:${port}${path}`);
        if (res.ok) ok(`GET ${path} -> ${res.status}`);
        else fail(`GET ${path} -> ${res.status}`);
      }
      server.close(() => resolvePromise());
    });
  });
}

if (failed) {
  console.error(`\n${failed} smoke check(s) failed`);
  process.exit(1);
}
console.log("\nweb smoke checks passed");
