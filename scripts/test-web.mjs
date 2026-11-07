#!/usr/bin/env node
// Automated web/WASM test: static checks, build artifact checks, HTTP smoke,
// and headless Chrome smoke (console/page errors, quad-storage mismatch, DOM bridge).
//
// Usage:
//   node scripts/test-web.mjs [--root dist] [--no-browser] [--port 0]
//   node scripts/test-web.mjs --build-dev   # build wasm dev + assemble dist first

import { spawn } from "node:child_process";
import { access, mkdir, readFile, stat, copyFile, readdir } from "node:fs/promises";
import { constants } from "node:fs";
import { createServer } from "node:http";
import { extname, join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const args = process.argv.slice(2);
const root = resolve(argValue("--root", "dist"));
const noBrowser = args.includes("--no-browser");
const buildDev = args.includes("--build-dev");
const preferredPort = Number(argValue("--port", "0"));

function argValue(name, fallback) {
  const i = args.indexOf(name);
  return i >= 0 && args[i + 1] ? args[i + 1] : fallback;
}

let failed = 0;
const failures = [];
function ok(msg) { console.log(`ok: ${msg}`); }
function fail(msg) { console.error(`fail: ${msg}`); failed++; failures.push(msg); }

async function run(cmd, cmdArgs, opts = {}) {
  return await new Promise((resolveRun, rejectRun) => {
    const child = spawn(cmd, cmdArgs, { stdio: "inherit", ...opts });
    child.on("error", rejectRun);
    child.on("exit", (code) => code === 0 ? resolveRun() : rejectRun(new Error(`${cmd} exited ${code}`)));
  });
}

async function assembleDevDist() {
  console.log("building dev wasm and assembling dist...");
  await run("cargo", ["build", "-p", "enzo-wasm", "--target", "wasm32-unknown-unknown"], { cwd: resolve(".") });
  await mkdir(join(root, "assets"), { recursive: true });
  await mkdir(join(root, "ui"), { recursive: true });
  await copyFile("target/wasm32-unknown-unknown/debug/enzo-wasm.wasm", join(root, "enzo-wasm.wasm"));
  for (const f of ["index.html", "mq_js_bundle.js", "sapp_jsutils.js", "quad-storage.js", "ui.css"]) {
    await copyFile(join("web", f), join(root, f));
  }
  for (const f of await readdir("web/ui")) {
    if (f.endsWith(".js")) await copyFile(join("web/ui", f), join(root, "ui", f));
  }
  await copyFile("web/quad-storage.js", join(root, "quad-storage.js")); // ensure fixed plugin
  // copy assets recursively (simple)
  await run("bash", ["-lc", `cp -r assets/. '${root}/assets/' && touch '${root}/.nojekyll'`], { cwd: resolve(".") });
  ok("dev dist assembled");
}

async function checkNodeSyntax() {
  const files = [
    "scripts/smoke-web.mjs",
    "scripts/test-web.mjs",
    "scripts/serve.mjs",
    "web/ui/main.js",
    "web/ui/bridge.js",
    "web/ui/mobile-controls.js",
    "web/ui/state.js",
    "web/ui/hud.js",
    "web/ui/menus.js",
  ];
  for (const file of files) {
    try {
      await run(process.execPath, ["--check", file], { cwd: resolve(".") });
      ok(`syntax ${file}`);
    } catch (e) {
      fail(`syntax ${file}: ${e.message}`);
    }
  }
}

async function checkArtifacts() {
  const required = [
    "index.html", "enzo-wasm.wasm", "mq_js_bundle.js", "sapp_jsutils.js",
    "quad-storage.js", "ui.css", "ui/main.js", "ui/bridge.js", "ui/hud.js", "ui/menus.js", "assets",
  ];
  for (const rel of required) {
    try {
      await access(join(root, rel), constants.R_OK);
      const info = await stat(join(root, rel));
      ok(`${rel}${info.isFile() ? ` (${info.size} bytes)` : "/"}`);
    } catch {
      fail(`missing ${rel}`);
    }
  }

  const qs = await readFile(join(root, "quad-storage.js"), "utf8");
  if (!/name:\s*["']quad_storage["']/.test(qs)) fail("quad-storage plugin name missing");
  else ok("quad-storage plugin name present");
  if (/version:\s*["']0\.1\.4["']/.test(qs)) fail("quad-storage version is stale 0.1.4 string");
  else ok("quad-storage version not stale 0.1.4");

  const html = await readFile(join(root, "index.html"), "utf8");
  if (html.includes('type="module"') && html.includes("ui/main.js")) ok("index.html loads modular UI");
  else fail("index.html missing modular UI entry");
}

const mime = {
  ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8", ".wasm": "application/wasm",
  ".png": "image/png", ".svg": "image/svg+xml",
};

function startServer() {
  return new Promise((resolveServer) => {
    const server = createServer(async (req, res) => {
      try {
        const url = new URL(req.url ?? "/", "http://localhost");
        const requested = url.pathname.endsWith("/") ? `${url.pathname}index.html` : url.pathname;
        const file = resolve(join(root, requested));
        if (!file.startsWith(root)) return res.writeHead(403).end();
        const body = await readFile(file);
        res.writeHead(200, { "Content-Type": mime[extname(file)] ?? "application/octet-stream", "Cache-Control": "no-store" });
        res.end(body);
      } catch {
        res.writeHead(404).end();
      }
    });
    server.listen(preferredPort, "127.0.0.1", () => resolveServer(server));
  });
}

async function httpSmoke(baseUrl) {
  for (const path of ["/", "/index.html", "/enzo-wasm.wasm", "/ui/main.js", "/quad-storage.js", "/mq_js_bundle.js"]) {
    const res = await fetch(`${baseUrl}${path}`);
    if (res.ok) ok(`GET ${path} -> ${res.status} ${res.headers.get("content-type") || ""}`);
    else fail(`GET ${path} -> ${res.status}`);
  }
}

async function cdpSmoke(baseUrl) {
  let chromePath = null;
  for (const p of ["/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/chromium", "/usr/bin/chromium-browser"]) {
    try { await access(p, constants.X_OK); chromePath = p; break; } catch {}
  }
  if (!chromePath) {
    console.log("skip: headless browser smoke (Chrome/Chromium not found)");
    return;
  }
  if (typeof WebSocket === "undefined") {
    console.log("skip: headless browser smoke (Node WebSocket unavailable)");
    return;
  }

  const debugPort = 9222 + Math.floor(Math.random() * 500);
  const profileDir = `/tmp/opencode/enzoballz-chrome-${Date.now()}`;
  const proc = spawn(chromePath, [
    "--headless=new",
    `--remote-debugging-port=${debugPort}`,
    `--user-data-dir=${profileDir}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    "--no-sandbox",
    "about:blank",
  ], { stdio: "ignore" });

  try {
    let wsUrl = null;
    for (let i = 0; i < 60; i++) {
      try {
        const res = await fetch(`http://127.0.0.1:${debugPort}/json/list`);
        if (res.ok) {
          const list = await res.json();
          const page = list.find((t) => t.type === "page" && t.webSocketDebuggerUrl);
          if (page) { wsUrl = page.webSocketDebuggerUrl; break; }
        }
      } catch {}
      await delay(250);
    }
    if (!wsUrl) {
      try {
        const res = await fetch(`http://127.0.0.1:${debugPort}/json/new?${encodeURIComponent("about:blank")}`, { method: "PUT" });
        if (res.ok) {
          const page = await res.json();
          if (page.webSocketDebuggerUrl) wsUrl = page.webSocketDebuggerUrl;
        }
      } catch {}
    }
    if (!wsUrl) {
      fail("Chrome page target not available");
      return;
    }

    const result = await runCdpSession(wsUrl, baseUrl);
    for (const msg of result.console) {
      const text = `[${msg.type}] ${msg.text}`;
      const isGenericResource404 = msg.type === "error" && /^Failed to load resource: the server responded with a status of 404/i.test(msg.text);
      if (isGenericResource404) {
        console.log(`console: ${text} (see network details)`);
      } else if (msg.type === "error" || /quad.storage|version mismatch|Plugin .* mismatch|Uncaught|panic/i.test(msg.text)) {
        fail(`console: ${text}`);
      } else if (text.trim()) {
        console.log(`console: ${text}`);
      }
    }
    for (const err of result.pageErrors) fail(`page error: ${err}`);
    for (const nf of (result.networkFailures || [])) {
      const url = nf.url || "(unknown)";
      if (/\/favicon\.ico(\?|$)/.test(url)) {
        console.log(`warn: benign network failure ${nf.status || nf.error} ${url}`);
      } else {
        fail(`network ${nf.status || nf.error || "failed"} ${url}`);
      }
    }
    if (result.bridgeOk) ok("window.EnzoBridge present");
    else fail("window.EnzoBridge missing");
    if (result.domUiOk) ok("DOM UI roots present");
    else fail("DOM UI roots missing");
    if (result.canvasOk) ok("canvas present");
    else fail("canvas missing");
    if (result.wasmReady) ok("wasm exports/ui state responding");
    else console.log("warn: wasm exports/ui state not confirmed (may still be loading)");
    if (result.menuFlowOk) ok("menu flow: Play -> Saves -> New Game/Load -> game starts");
    else fail(`menu flow: ${result.menuFlowError || "failed"}${result.menuFlowState ? ` (state scene=${result.menuFlowState.scene} menu=${JSON.stringify(result.menuFlowState.menu)})` : ""}`);
    if (!result.quadStorageMismatch) ok("no quad-storage version mismatch detected");
  } catch (e) {
    fail(`headless browser smoke: ${e.message}`);
  } finally {
    proc.kill("SIGTERM");
    await delay(200);
    try { proc.kill("SIGKILL"); } catch {}
  }
}

function runCdpSession(wsUrl, baseUrl) {
  return new Promise((resolveSession, rejectSession) => {
    const ws = new WebSocket(wsUrl);
    let id = 0;
    const pending = new Map();
    const consoleMsgs = [];
    const pageErrors = [];
    const networkFailures = [];
    const requests = new Map();
    let quadStorageMismatch = false;

    function send(method, params = {}) {
      const msgId = ++id;
      ws.send(JSON.stringify({ id: msgId, method, params }));
      return new Promise((res, rej) => pending.set(msgId, { res, rej }));
    }

    ws.addEventListener("message", async (ev) => {
      const msg = JSON.parse(String(ev.data));
      if (msg.id && pending.has(msg.id)) {
        const { res, rej } = pending.get(msg.id);
        pending.delete(msg.id);
        if (msg.error) rej(new Error(msg.error.message));
        else res(msg.result);
        return;
      }
      if (msg.method === "Runtime.consoleAPICalled") {
        const text = (msg.params.args || []).map(a => a.value ?? a.description ?? a.type ?? "").join(" ");
        if (text) consoleMsgs.push({ type: msg.params.type, text });
        if (/quad.storage|version mismatch|Plugin .*quad_storage.*mismatch/i.test(text)) quadStorageMismatch = true;
      }
      if (msg.method === "Runtime.exceptionThrown") {
        const text = msg.params.exceptionDetails?.text || msg.params.exceptionDetails?.exception?.description || "exception";
        pageErrors.push(text);
        if (/quad.storage|version mismatch/i.test(text)) quadStorageMismatch = true;
      }
      if (msg.method === "Log.entryAdded") {
        const text = msg.params.entry?.text || "";
        if (msg.params.entry?.level === "error" && text) consoleMsgs.push({ type: "error", text });
        if (/quad.storage|version mismatch/i.test(text)) quadStorageMismatch = true;
      }
      if (msg.method === "Network.requestWillBeSent") {
        requests.set(msg.params.requestId, msg.params.request?.url || "");
      }
      if (msg.method === "Network.responseReceived") {
        const status = msg.params.response?.status || 0;
        if (status >= 400) {
          networkFailures.push({ url: msg.params.response?.url || requests.get(msg.params.requestId) || "", status, type: msg.params.type });
        }
      }
      if (msg.method === "Network.loadingFailed") {
        networkFailures.push({ url: requests.get(msg.params.requestId) || "", status: 0, error: msg.params.errorText, canceled: msg.params.canceled });
      }
    });

    ws.addEventListener("open", async () => {
      try {
        await send("Page.enable");
        await send("Runtime.enable");
        await send("Log.enable");
        await send("Network.enable");
        await send("Page.navigate", { url: baseUrl });
        await delay(7000); // allow wasm load/init (debug wasm is large)
        async function evalJson(expression) {
          const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
          if (r.exceptionDetails) throw new Error(r.exceptionDetails.text || "eval exception");
          try { return JSON.parse(r.result?.value ?? "null"); } catch { return r.result?.value; }
        }
        async function waitFor(label, fnSource, timeoutMs = 12000) {
          const start = Date.now();
          while (Date.now() - start < timeoutMs) {
            const ok = await evalJson(`(${fnSource})()`);
            if (ok) return true;
            await delay(150);
          }
          throw new Error(`timeout waiting for ${label}`);
        }
        async function readUiState() {
          return await evalJson(`(() => {
            try {
              if (!globalThis.wasm_exports || !globalThis.wasm_exports.enzo_ui_state_ptr) return null;
              const ptr = globalThis.wasm_exports.enzo_ui_state_ptr();
              const len = globalThis.wasm_exports.enzo_ui_state_len();
              if (!ptr || !len || !globalThis.wasm_memory) return null;
              return JSON.parse(new TextDecoder().decode(new Uint8Array(globalThis.wasm_memory.buffer, ptr, len)));
            } catch (e) { return { __error: String(e) }; }
          })()`);
        }

        const evalResult = await send("Runtime.evaluate", {
          expression: `JSON.stringify((() => {
            const bridge = globalThis.EnzoBridge;
            let wasmReady = false;
            try { wasmReady = !!(globalThis.wasm_exports && typeof globalThis.wasm_exports.enzo_action_down === 'function'); } catch {}
            return {
              bridgeOk: !!bridge && typeof bridge.actionDown === 'function',
              domUiOk: !!document.getElementById('dom-ui') && !!document.getElementById('dom-controls-root'),
              canvasOk: !!document.querySelector('canvas'),
              wasmReady,
              title: document.title
            };
          })())`,
          returnByValue: true,
        });
        let info = {};
        try { info = JSON.parse(evalResult.result?.value || "{}"); } catch {}

        // Functional flow: Play -> Saves -> New Game starts game.
        try {
          await waitFor("wasm ui state", `() => !!(globalThis.wasm_exports && globalThis.wasm_exports.enzo_ui_state_ptr)`);
          await waitFor("menu ui state", `() => {
            try {
              const p = globalThis.wasm_exports.enzo_ui_state_ptr(), l = globalThis.wasm_exports.enzo_ui_state_len();
              if (!p || !l) return false;
              const s = JSON.parse(new TextDecoder().decode(new Uint8Array(globalThis.wasm_memory.buffer, p, l)));
              return !!s && !!s.menu;
            } catch { return false; }
          }`, 15000);
          // Click Play (opens Saves)
          await evalJson(`(() => {
            const btn = [...document.querySelectorAll('#dom-menu-root button')].find(b => /play|jogar/i.test(b.textContent||''));
            if (btn) { btn.click(); return true; }
            return false;
          })()`);
          await waitFor("saves screen", `() => {
            try {
              const p = globalThis.wasm_exports.enzo_ui_state_ptr(), l = globalThis.wasm_exports.enzo_ui_state_len();
              const s = JSON.parse(new TextDecoder().decode(new Uint8Array(globalThis.wasm_memory.buffer, p, l)));
              return s && s.menu && s.menu.screen === 'saves';
            } catch { return false; }
          }`);
          // Click New Game / Load on first slot
          await evalJson(`(() => {
            const btn = [...document.querySelectorAll('#dom-menu-root button')].find(b => /new game|novo jogo|load|carregar/i.test(b.textContent||''));
            if (btn) { btn.click(); return true; }
            return false;
          })()`);
          await waitFor("game scene", `() => {
            try {
              const p = globalThis.wasm_exports.enzo_ui_state_ptr(), l = globalThis.wasm_exports.enzo_ui_state_len();
              const s = JSON.parse(new TextDecoder().decode(new Uint8Array(globalThis.wasm_memory.buffer, p, l)));
              return s && s.scene === 'game';
            } catch { return false; }
          }`, 15000);
          info.menuFlowOk = true;
        } catch (e) {
          info.menuFlowOk = false;
          info.menuFlowError = String(e && e.message ? e.message : e);
          try { info.menuFlowState = await readUiState(); } catch {}
        }

        ws.close();
        resolveSession({ console: consoleMsgs, pageErrors, networkFailures, quadStorageMismatch, ...info });
      } catch (e) {
        try { ws.close(); } catch {}
        rejectSession(e);
      }
    });
    ws.addEventListener("error", () => rejectSession(new Error("CDP websocket error")));
  });
}

async function main() {
  if (buildDev) await assembleDevDist();
  await checkNodeSyntax();
  await checkArtifacts();
  const server = await startServer();
  const { port } = server.address();
  const baseUrl = `http://127.0.0.1:${port}`;
  ok(`test server ${baseUrl}`);
  try {
    await httpSmoke(baseUrl);
    if (!noBrowser) await cdpSmoke(baseUrl);
    else console.log("skip: browser smoke (--no-browser)");
  } finally {
    server.close();
  }

  if (failed) {
    console.error(`\n${failed} web test(s) failed`);
    process.exit(1);
  }
  console.log("\nweb tests passed");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
