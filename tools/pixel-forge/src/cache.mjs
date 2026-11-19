import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
export const TOOL_CACHE_PATH = join(ROOT, 'tools/pixel-forge/.build-cache.json');
export const TOOL_VERSION = '1';

export function sha256(bufferOrString) {
  return createHash('sha256').update(bufferOrString).digest('hex');
}

export function hashFile(path) {
  try {
    return sha256(readFileSync(path));
  } catch {
    return `missing:${path}`;
  }
}

export function hashFiles(paths) {
  const h = createHash('sha256');
  for (const path of [...paths].sort()) {
    h.update(path);
    h.update('\0');
    h.update(hashFile(path));
    h.update('\0');
  }
  return h.digest('hex');
}

export function fingerprint(parts) {
  const h = createHash('sha256');
  h.update(`v${TOOL_VERSION}\0`);
  const flat = Array.isArray(parts) ? parts : [parts];
  for (const part of flat) {
    if (part && typeof part === 'object' && part.kind === 'file') h.update(hashFile(part.path));
    else if (part && typeof part === 'object' && part.kind === 'files') h.update(hashFiles(part.paths));
    else h.update(typeof part === 'string' ? part : JSON.stringify(part));
    h.update('\0');
  }
  return h.digest('hex');
}

export function ensureDir(path) {
  mkdirSync(dirname(path), { recursive: true });
}

function sameContent(path, buffer) {
  if (!existsSync(path)) return false;
  try {
    const current = readFileSync(path);
    return Buffer.isBuffer(buffer) ? current.equals(buffer) : current.equals(Buffer.from(buffer));
  } catch {
    return false;
  }
}

export function writeBufferIfChanged(path, buffer, stats = null) {
  const buf = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
  if (sameContent(path, buf)) {
    if (stats) stats.unchanged += 1;
    return false;
  }
  ensureDir(path);
  writeFileSync(path, buf);
  if (stats) stats.written += 1;
  return true;
}

export function writeTextIfChanged(path, text, stats = null) {
  return writeBufferIfChanged(path, text, stats);
}

export function writeJsonIfChanged(path, value, stats = null) {
  return writeTextIfChanged(path, JSON.stringify(value, null, 2) + '\n', stats);
}

export function loadCache() {
  if (!existsSync(TOOL_CACHE_PATH)) return { version: 1, scopes: {} };
  try {
    const data = JSON.parse(readFileSync(TOOL_CACHE_PATH, 'utf8'));
    if (!data || typeof data !== 'object' || !data.scopes) return { version: 1, scopes: {} };
    return data;
  } catch {
    return { version: 1, scopes: {} };
  }
}

export function saveCache(cache) {
  ensureDir(TOOL_CACHE_PATH);
  writeFileSync(TOOL_CACHE_PATH, JSON.stringify(cache, null, 2) + '\n');
}

export function scopeCached(cache, scope, fp, outputs, force = false) {
  if (force) return false;
  const entry = cache.scopes?.[scope];
  if (!entry || entry.fingerprint !== fp) return false;
  return outputs.every((p) => existsSync(p));
}

export function updateScopeCache(cache, scope, fp, outputs) {
  cache.version = 1;
  cache.scopes ||= {};
  cache.scopes[scope] = { fingerprint: fp, outputs };
}

export function createStats() {
  return { written: 0, unchanged: 0, skippedScopes: 0 };
}

export function formatStats(stats) {
  const parts = [`written ${stats.written}`, `unchanged ${stats.unchanged}`];
  if (stats.skippedScopes) parts.push(`skipped scopes ${stats.skippedScopes}`);
  return parts.join(', ');
}


export function writePngIfChanged(path, canvas, stats = null, encodePng) {
  return writeBufferIfChanged(path, encodePng(canvas), stats);
}
