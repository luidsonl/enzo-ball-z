// Slime entity sprite generation/validation (separate entity code).
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from '../png.mjs';
import { P } from '../palette.mjs';
import {
  createStats,
  fingerprint,
  loadCache,
  saveCache,
  scopeCached,
  updateScopeCache,
  writeJsonIfChanged,
  writePngIfChanged,
} from '../cache.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../../..');

export function drawSlime(c, ox, frame) {
  const hurt = frame === 'hurt';
  const body = hurt ? '#ff9aa8' : '#66d17a';
  const dark = hurt ? '#d85a6a' : '#3f9e57';
  const h = frame === 'stretch' ? 12 : frame === 'walk_b' ? 14 : 15;
  const w = frame === 'stretch' ? 22 : 18;
  const x = ox + (32 - w) / 2;
  const y = 30 - h;
  c.rect(x, y, w, h, body);
  c.rect(x, y + h - 3, w, 3, dark);
  c.rect(x + w * 0.25, y + h * 0.35, 3, 4, P.outline);
  c.rect(x + w * 0.6, y + h * 0.35, 3, 4, P.outline);
  c.rect(x + w * 0.35, y + h * 0.65, w * 0.3, 2, '#ffffff66');
}

export function buildSlimeAssets(opts = {}) {
  const force = opts.force ?? false;
  const stats = opts.stats ?? createStats();
  const cache = opts.cache ?? loadCache();
  const slimeDir = join(ROOT, 'assets/entities/slime');
  const outputs = [
    join(slimeDir, 'sheet.png'),
    join(slimeDir, 'atlas.json'),
    join(slimeDir, 'manifest.json'),
  ];
  const fp = fingerprint([
    { kind: 'files', paths: [
      join(ROOT, 'tools/pixel-forge/src/entities/slime.mjs'),
      join(ROOT, 'tools/pixel-forge/src/entities.mjs'),
      join(ROOT, 'tools/pixel-forge/src/palette.mjs'),
      join(ROOT, 'tools/pixel-forge/src/png.mjs'),
    ]},
  ]);
  if (scopeCached(cache, 'entity:slime', fp, outputs, force)) {
    stats.skippedScopes += 1;
    if (!opts.cache) saveCache(cache);
    return ['assets/entities/slime/sheet.png', 'assets/entities/slime/atlas.json'];
  }

  const slimeFrames = [
    { name: 'walk_a', draw: (c, x) => drawSlime(c, x, 'walk_a') },
    { name: 'walk_b', draw: (c, x) => drawSlime(c, x, 'walk_b') },
    { name: 'stretch', draw: (c, x) => drawSlime(c, x, 'stretch') },
    { name: 'hurt', draw: (c, x) => drawSlime(c, x, 'hurt') },
  ];
  const slimeSheet = new Canvas(32 * slimeFrames.length, 32);
  slimeFrames.forEach((f, i) => f.draw(slimeSheet, i * 32));
  mkdirSync(slimeDir, { recursive: true });
  writePngIfChanged(join(slimeDir, 'sheet.png'), slimeSheet, stats, encodePng);
  const slimeAtlas = {
    image: 'sheet.png', cell: { w: 32, h: 32 },
    frames: Object.fromEntries(slimeFrames.map((f, i) => [f.name, { x: i * 32, y: 0, w: 32, h: 32 }])),
    clips: { walk: ['walk_a', 'walk_b'], stretch: ['stretch'], hurt: ['hurt'] },
  };
  writeJsonIfChanged(join(slimeDir, 'atlas.json'), slimeAtlas, stats);
  writeJsonIfChanged(join(slimeDir, 'manifest.json'), {
    framework: 'pixel-forge', id: 'slime', sheet: 'sheet.png', atlas: 'atlas.json', clips: Object.keys(slimeAtlas.clips),
  }, stats);

  updateScopeCache(cache, 'entity:slime', fp, outputs);
  if (!opts.cache) saveCache(cache);
  return ['assets/entities/slime/sheet.png', 'assets/entities/slime/atlas.json'];
}

export function validateSlimeAssets(errors = []) {
  for (const name of ['sheet.png', 'atlas.json', 'manifest.json']) {
    const p = join(ROOT, 'assets/entities/slime', name);
    if (!existsSync(p)) errors.push(`entity slime: missing ${name}`);
  }
  const sheet = join(ROOT, 'assets/entities/slime/sheet.png');
  if (existsSync(sheet)) {
    try {
      const b = readFileSync(sheet);
      if (b.length < 24 || b[0] !== 137 || b[1] !== 80) errors.push('entity slime: invalid PNG sheet.png');
    } catch {
      errors.push('entity slime: invalid PNG sheet.png');
    }
  }
  return errors;
}
