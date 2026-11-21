import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';
import { HUMANOID_FRAMES } from './entities.mjs';
import {
  createStats,
  fingerprint,
  loadCache,
  saveCache,
  scopeCached,
  updateScopeCache,
  writeJsonIfChanged,
  writePngIfChanged,
} from './cache.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
export const CORE_ITEMS_PATH = join(ROOT, 'assets/definitions/generated/items.json');

const VISUAL_HINTS = {
  sword: { bg: '#3a4a6b', fg: '#d8e8ff', world: 'blade', equipped: 'blade', layer: 'weapon' },
  bow: { bg: '#5a3a22', fg: P.woodLight, world: 'bow', equipped: 'bow', layer: 'weapon' },
  armor: { bg: '#555b66', fg: '#cfd8e6', world: 'plate', equipped: 'plate', layer: 'armor' },
  amulet: { bg: '#2f5a4a', fg: '#8fffd8', world: 'gem', equipped: 'gem', layer: 'accessory' },
  potion: { bg: '#6b1f3a', fg: '#ff8fb8', world: 'bottle', equipped: 'bottle', layer: null },
  nutella_gun: { bg: P.crateDark, fg: P.woodLight, world: 'gun', equipped: 'gun', layer: 'weapon' },
  sword_67: { bg: '#2a355f', fg: '#7cf6ff', accent: '#ffd84d', world: 'blade67', equipped: 'blade67', layer: 'weapon' },
  cap_67: { bg: '#2a355f', fg: '#ffd84d', accent: '#7cf6ff', world: 'cap', equipped: 'cap', layer: 'head' },
  shirt_67: { bg: '#2a355f', fg: '#7cf6ff', accent: '#ffd84d', world: 'shirt', equipped: 'shirt', layer: 'armor' },
  sneakers_67: { bg: '#2a355f', fg: '#e8eefc', accent: '#ffd84d', world: 'shoes', equipped: 'shoes', layer: 'feet' },
  enzotella_big: { bg: '#4a2a16', fg: '#7a4a2a', world: 'enzotella_pot', equipped: 'enzotella_pot', layer: null },
  enzotella_medium: { bg: '#4a2a16', fg: '#7a4a2a', world: 'enzotella_pot', equipped: 'enzotella_pot', layer: null },
  enzotella_small: { bg: '#4a2a16', fg: '#7a4a2a', world: 'enzotella_pot', equipped: 'enzotella_pot', layer: null },
};

function loadContract() {
  if (existsSync(CORE_ITEMS_PATH)) return JSON.parse(readFileSync(CORE_ITEMS_PATH, 'utf8'));
  return { source: 'fallback', items: Object.keys(VISUAL_HINTS).map((id) => ({ id, visual: {
    icon: `assets/items/${id}/icon.png`,
    world: `assets/items/${id}/world.png`,
    equipped: `assets/items/${id}/equipped.png`,
  }})) };
}

function draw67Mark(c, x, y, fg, accent, s = 1) {
  s = Math.max(1, Math.round(s));
  // "6"
  c.rect(x, y, s, 6 * s, fg);
  c.rect(x + s, y, 3 * s, s, fg);
  c.rect(x + 3 * s, y + s, s, 2 * s, fg);
  c.rect(x + s, y + 3 * s, 2 * s, s, accent || fg);
  c.rect(x + s, y + 5 * s, 3 * s, s, fg);
  c.rect(x + 3 * s, y + 4 * s, s, s, fg);
  // "7"
  const x7 = x + 5 * s;
  c.rect(x7, y, 4 * s, s, fg);
  c.rect(x7 + 3 * s, y + s, s, 4 * s, fg);
  c.rect(x7, y + 5 * s, 4 * s, s, fg);
  c.rect(x7, y + 3 * s, 3 * s, s, accent || fg);
}

function pose(frame) {
  if (frame.startsWith('walk_')) return { y: 0, step: frame==='walk_1'||frame==='walk_3'?1:frame==='walk_2'?-1:0, crouch:false };
  if (frame.startsWith('run_')) return { y: 0, step: frame==='run_1'||frame==='run_3'?2:frame==='run_2'?-2:0, crouch:false };
  if (frame === 'idle_1') return { y: -1, step: 0, crouch:false };
  if (frame === 'crouch_0') return { y: 2, step: 0, crouch:true };
  if (frame === 'jump_rise_0') return { y: -3, step: 0, crouch:false };
  if (frame === 'jump_fall_0' || frame === 'dash_0') return { y: -1, step: 1, crouch:false };
  if (frame.startsWith('fly_')) return { y: -2, step: frame==='fly_1'?1:0, crouch:false };
  if (frame.startsWith('swim_')) return { y: -1, step: frame==='swim_1'?1:-1, crouch:false };
  if (frame === 'dying_0' || frame === 'dead_0') return { y: 2, step: 0, crouch:true };
  return { y: 0, step: 0, crouch:false };
}

function drawChocolatePot16(c, ox = 0, oy = 0) {
  // 16x16 chocolate pot with "67" on label
  // lid
  c.rect(ox + 3, oy + 1, 10, 3, '#c7373d');
  c.rect(ox + 4, oy + 1, 8, 1, '#e85a5f');
  c.rect(ox + 3, oy + 3, 10, 1, '#8f1f24');
  // neck
  c.rect(ox + 4, oy + 4, 8, 1, '#9b5b34');
  // pot body
  c.rect(ox + 3, oy + 5, 10, 9, '#6b3a1c');
  c.rect(ox + 4, oy + 5, 8, 8, '#7a4422');
  c.rect(ox + 3, oy + 13, 10, 1, '#4a2812');
  c.rect(ox + 3, oy + 5, 1, 8, '#4a2812');
  c.rect(ox + 12, oy + 5, 1, 8, '#4a2812');
  // chocolate at top inside
  c.rect(ox + 4, oy + 5, 8, 2, '#4a2814');
  c.rect(ox + 5, oy + 6, 6, 1, '#5a3118');
  // label
  c.rect(ox + 4, oy + 8, 8, 4, '#f3e7d3');
  c.rect(ox + 4, oy + 8, 8, 1, '#ffffff');
  c.rect(ox + 4, oy + 11, 8, 1, '#cdbb9b');
  // "67"
  // 6
  c.rect(ox + 5, oy + 9, 1, 2, '#5a2f18');
  c.rect(ox + 6, oy + 9, 1, 1, '#5a2f18');
  c.rect(ox + 6, oy + 10, 1, 1, '#c83a3a');
  // 7
  c.rect(ox + 8, oy + 9, 3, 1, '#5a2f18');
  c.rect(ox + 10, oy + 10, 1, 1, '#5a2f18');
  c.rect(ox + 8, oy + 10, 1, 1, '#c83a3a');
  // highlight/shadow
  c.rect(ox + 4, oy + 7, 2, 5, '#8b533044');
}

function drawChocolatePot(c, x, y, w, h) {
  if (w <= 16 && h <= 16) {
    drawChocolatePot16(c, Math.round(x), Math.round(y));
    return;
  }
  if (w <= 20 && h <= 20) {
    // draw centered 16x16 pot in small canvases
    drawChocolatePot16(c, Math.round(x + (w - 16) / 2), Math.round(y + (h - 16) / 2));
    return;
  }
  // larger canvases: draw scaled 16x16 pixel art for crisp reuse
  const scale = Math.max(1, Math.floor(Math.min(w, h) / 16));
  const sprite = new Canvas(16, 16);
  drawChocolatePot16(sprite, 0, 0);
  const sw = 16 * scale;
  const sh = 16 * scale;
  const ox = Math.round(x + (w - sw) / 2);
  const oy = Math.round(y + (h - sh) / 2);
  for (let py = 0; py < 16; py += 1) {
    for (let px = 0; px < 16; px += 1) {
      const i = (py * 16 + px) * 4;
      const a = sprite.data[i + 3];
      if (a === 0) continue;
      c.rect(ox + px * scale, oy + py * scale, scale, scale, [
        sprite.data[i],
        sprite.data[i + 1],
        sprite.data[i + 2],
        a,
      ]);
    }
  }
}

function icon(bg, fg, accent = null, worldKind = null) {
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline); c.rect(1, 1, 30, 30, bg); c.rect(2, 2, 28, 2, '#ffffff33');
  if (worldKind === 'enzotella_pot') {
    drawChocolatePot(c, 0, 0, 32, 32);
  } else {
    c.rect(10, 8, 12, 16, fg); c.rect(12, 10, 8, 12, bg); c.rect(15, 6, 2, 20, '#ffffff55');
  }
  if (accent) draw67Mark(c, 11, 22, fg, accent);
  return c;
}

function worldSprite(kind, fg, accent = null) {
  if (kind === 'enzotella_pot') {
    const c = new Canvas(16, 16);
    drawChocolatePot(c, 0, 0, 16, 16);
    return c;
  }
  const c = new Canvas(20, 20);
  if (kind === 'blade' || kind === 'blade67') {
    c.rect(9, 2, 2, 14, fg); c.rect(6, 14, 8, 2, P.wood); c.rect(9, 16, 2, 2, P.woodLight);
    if (kind === 'blade67' && accent) c.rect(8, 5, 4, 2, accent);
  } else if (kind === 'bow') {
    c.rect(8, 3, 2, 14, P.wood); c.rect(10, 4, 3, 2, P.woodLight); c.rect(10, 14, 3, 2, P.woodLight); c.rect(12, 6, 1, 8, '#ffffff66');
  } else if (kind === 'plate') {
    c.rect(5, 4, 10, 12, fg); c.rect(6, 5, 8, 10, '#8a95a8'); c.rect(9, 6, 2, 8, '#ffffff44');
  } else if (kind === 'gem') {
    c.rect(9, 4, 2, 2, fg); c.rect(8, 6, 4, 6, fg); c.rect(9, 12, 2, 3, P.gold); c.rect(9, 7, 2, 4, '#ffffff55');
  } else if (kind === 'bottle') {
    c.rect(8, 7, 4, 9, fg); c.rect(9, 4, 2, 3, '#d8f0ff'); c.rect(8, 6, 4, 2, P.outline); c.rect(9, 9, 2, 5, '#ffffff44');
  } else if (kind === 'gun') {
    c.rect(5, 9, 10, 4, P.crate); c.rect(13, 8, 3, 3, P.crateDark); c.rect(7, 7, 3, 2, P.wood); c.rect(8, 10, 5, 2, '#ffffff33');
  } else if (kind === 'cap') {
    c.rect(4, 10, 12, 4, fg); c.rect(6, 7, 8, 4, fg); c.rect(13, 11, 4, 2, accent || '#ffffff88');
  } else if (kind === 'shirt') {
    c.rect(6, 5, 8, 10, fg); c.rect(4, 6, 2, 5, fg); c.rect(14, 6, 2, 5, fg); c.rect(8, 8, 4, 4, accent || '#ffffff33');
  } else if (kind === 'shoes') {
    c.rect(4, 12, 6, 4, fg); c.rect(10, 12, 6, 4, fg); c.rect(5, 13, 4, 2, accent || '#ffffff55'); c.rect(11, 13, 4, 2, accent || '#ffffff55');
  } else { c.rect(6, 6, 8, 8, fg); }
  if (accent && (kind === 'cap' || kind === 'shirt' || kind === 'shoes')) draw67Mark(c, 5, 1, fg, accent);
  return c;
}

function drawEquippedAt(c, ox, frame, kind, fg, accent = null) {
  const p = pose(frame);
  const y = p.y; const bodyY = 14 + y; const headY = 6 + y; const fy = 29;
  const attack = frame.startsWith('attack_') || frame === 'kick_0' || frame === 'cast_0';
  const armX = frame === 'attack_3_0' ? 25 : frame === 'attack_2_0' ? 24 : frame === 'attack_1_0' || frame === 'cast_0' ? 23 : 22;
  if (kind === 'blade' || kind === 'blade67') {
    c.rect(ox + armX, bodyY - 2 + (attack ? 0 : 2), 2, 14, fg);
    c.rect(ox + armX - 2, bodyY + 10, 6, 2, P.wood);
    if (kind === 'blade67' && accent) c.rect(ox + armX - 1, bodyY + 2, 4, 2, accent);
  } else if (kind === 'bow') {
    c.rect(ox + armX, bodyY, 2, 14, P.wood); c.rect(ox + armX + 2, bodyY + 2, 2, 2, P.woodLight); c.rect(ox + armX + 2, bodyY + 10, 2, 2, P.woodLight);
  } else if (kind === 'gun') {
    c.rect(ox + armX - 1, bodyY + 2, 8, 4, P.crate); c.rect(ox + armX + 6, bodyY + 1, 3, 3, P.crateDark);
  } else if (kind === 'cap') {
    c.rect(ox + 9, headY, 14, 4, fg); c.rect(ox + 7, headY + 3, 18, 3, fg);
    if (accent) c.rect(ox + 20, headY + 4, 5, 1, accent);
  } else if (kind === 'shirt' || kind === 'plate') {
    c.rect(ox + 10, bodyY, 12, p.crouch ? 7 : 10, fg);
    c.rect(ox + 7, bodyY + 1, 3, p.crouch ? 4 : 7, fg);
    c.rect(ox + 22, bodyY + 1, 3, p.crouch ? 4 : 7, fg);
    if (accent) c.rect(ox + 13, bodyY + 2, 6, 3, accent);
  } else if (kind === 'shoes') {
    if (frame !== 'dying_0' && frame !== 'dead_0') {
      c.rect(ox + 9, fy, 6, 3, fg); c.rect(ox + 17, fy, 6, 3, fg);
      if (accent) { c.rect(ox + 10, fy + 1, 4, 1, accent); c.rect(ox + 18, fy + 1, 4, 1, accent); }
    }
  } else if (kind === 'gem') {
    c.rect(ox + 15, bodyY + 9, 2, 2, fg);
  }
}

function equippedSprite(kind, fg, accent = null) {
  const c = new Canvas(32, 32);
  drawEquippedAt(c, 0, 'idle_0', kind, fg, accent);
  return c;
}

function humanoidOverlaySheet(kind, fg, accent = null) {
  const c = new Canvas(32 * HUMANOID_FRAMES.length, 32);
  HUMANOID_FRAMES.forEach((frame, i) => drawEquippedAt(c, i * 32, frame, kind, fg, accent));
  return c;
}

export function buildItemAssets(opts = {}) {
  const force = opts.force ?? false;
  const stats = opts.stats ?? createStats();
  const cache = opts.cache ?? loadCache();
  const contract = loadContract();
  const files = [];
  const outputs = [];
  for (const item of contract.items || []) {
    const hint = VISUAL_HINTS[item.id] || { bg: '#444', fg: '#ddd', world: 'box', equipped: 'box', layer: null };
    const visual = item.visual || {};
    const dir = join(ROOT, 'assets/items', item.id);
    for (const slot of ['icon', 'world', 'equipped']) {
      const rel = visual[slot] || `assets/items/${item.id}/${slot}.png`;
      files.push(rel);
      outputs.push(join(ROOT, rel));
    }
    if (hint.layer || item.slot) {
      const layerRel = `assets/items/${item.id}/layers/humanoid_overlay.png`;
      files.push(layerRel);
      outputs.push(join(ROOT, layerRel));
    }
    outputs.push(join(dir, 'manifest.json'));
  }
  const fp = fingerprint([
    { kind: 'file', path: CORE_ITEMS_PATH },
    { kind: 'files', paths: [
      join(ROOT, 'tools/pixel-forge/src/items.mjs'),
      join(ROOT, 'tools/pixel-forge/src/entities.mjs'),
      join(ROOT, 'tools/pixel-forge/src/entities/humanoid.mjs'),
      join(ROOT, 'tools/pixel-forge/src/palette.mjs'),
      join(ROOT, 'tools/pixel-forge/src/png.mjs'),
    ]},
  ]);
  if (scopeCached(cache, 'items', fp, outputs, force)) {
    stats.skippedScopes += 1;
    if (!opts.cache) saveCache(cache);
    return files;
  }

  for (const item of contract.items || []) {
    const hint = VISUAL_HINTS[item.id] || { bg: '#444', fg: '#ddd', world: 'box', equipped: 'box', layer: null };
    const visual = item.visual || {};
    const dir = join(ROOT, 'assets/items', item.id);
    mkdirSync(join(dir, 'layers'), { recursive: true });
    const assets = [
      ['icon', icon(hint.bg, hint.fg, hint.accent || null, hint.world)],
      ['world', worldSprite(hint.world, hint.fg, hint.accent || null)],
      ['equipped', equippedSprite(hint.equipped || hint.world, hint.fg, hint.accent || null)],
    ];
    for (const [slot, canvas] of assets) {
      const rel = visual[slot] || `assets/items/${item.id}/${slot}.png`;
      const abs = join(ROOT, rel);
      writePngIfChanged(abs, canvas, stats, encodePng);
    }
    if (hint.layer || item.slot) {
      const layerRel = `assets/items/${item.id}/layers/humanoid_overlay.png`;
      const layerAbs = join(ROOT, layerRel);
      writePngIfChanged(layerAbs, humanoidOverlaySheet(hint.equipped || hint.world, hint.fg, hint.accent || null), stats, encodePng);
    }
    writeJsonIfChanged(join(dir, 'manifest.json'), {
      framework: 'pixel-forge',
      id: item.id,
      kind: item.kind,
      category: item.category,
      slot: item.slot,
      contract: 'assets/definitions/generated/items.json',
      assets: {
        icon: basename(visual.icon || 'icon.png'),
        world: basename(visual.world || 'world.png'),
        equipped: basename(visual.equipped || 'equipped.png'),
        humanoid_overlay: (hint.layer || item.slot) ? 'layers/humanoid_overlay.png' : null,
      },
    }, stats);
  }

  updateScopeCache(cache, 'items', fp, outputs);
  if (!opts.cache) saveCache(cache);
  return files;
}

export function validateItemAssets() {
  const contract = loadContract();
  const errors = [];
  for (const item of contract.items || []) {
    const visual = item.visual || {};
    for (const slot of ['icon', 'world']) {
      const rel = visual[slot] || `assets/items/${item.id}/${slot}.png`;
      const abs = join(ROOT, rel);
      if (!existsSync(abs)) errors.push(`item ${item.id}: missing ${slot} -> ${rel}`);
      else {
        const b = readFileSync(abs);
        if (b.length < 8 || b[0] !== 137 || b[1] !== 80) errors.push(`item ${item.id}: invalid PNG ${rel}`);
      }
    }
  }
  return errors;
}
