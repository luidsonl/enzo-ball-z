#!/usr/bin/env node
// Servidor estático mínimo para o build WASM. Sem dependências.
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join, normalize, resolve } from "node:path";

const root = resolve(process.argv.includes("--root")
  ? process.argv[process.argv.indexOf("--root") + 1]
  : "dist");
const port = Number(process.argv.includes("--port")
  ? process.argv[process.argv.indexOf("--port") + 1]
  : process.env.PORT ?? 8080);

const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
};

createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? "/", "http://localhost");
    const requested = url.pathname.endsWith("/")
      ? `${url.pathname}index.html`
      : url.pathname;
    const file = normalize(join(root, requested));

    if (!file.startsWith(root)) {
      res.writeHead(403).end("forbidden");
      return;
    }

    const info = await stat(file).catch(() => null);
    if (!info || !info.isFile()) {
      res.writeHead(404).end("not found");
      return;
    }

    const body = await readFile(file);
    res.writeHead(200, {
      "Content-Type": mime[extname(file)] ?? "application/octet-stream",
      "Cache-Control": "no-store",
      "Content-Length": body.length,
    });
    res.end(body);
  } catch (error) {
    res.writeHead(500).end(String(error));
  }
}).listen(port, "127.0.0.1", () => {
  console.log(`EnzoBallZ em http://127.0.0.1:${port}`);
});
