import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
export const CORE_ITEMS_PATH = join(ROOT, 'assets/definitions/generated/items.json');

const VISUAL_HINTS = {
  sword: { bg: '#3a4a6b', fg: '#d8e8ff', world: 'blade', equipped: 'blade' },
  bow: { bg: '#5a3a22', fg: P.woodLight, world: 'bow', equipped: 'bow' },
  armor: { bg: '#555b66', fg: '#cfd8e6', world: 'plate', equipped: 'plate' },
  amulet: { bg: '#2f5a4a', fg: '#8fffd8', world: 'gem', equipped: 'gem' },
  potion: { bg: '#6b1f3a', fg: '#ff8fb8', world: 'bottle', equipped: 'bottle' },
  nutella_gun: { bg: P.crateDark, fg: P.woodLight, world: 'gun', equipped: 'gun' },
  sword_67: { bg: '#2a355f', fg: '#7cf6ff', accent: '#ffd84d', world: 'blade67', equipped: 'blade67' },
  cap_67: { bg: '#2a355f', fg: '#ffd84d', accent: '#7cf6ff', world: 'cap', equipped: 'cap' },
  shirt_67: { bg: '#2a355f', fg: '#7cf6ff', accent: '#ffd84d', world: 'shirt', equipped: 'shirt' },
  sneakers_67: { bg: '#2a355f', fg: '#e8eefc', accent: '#ffd84d', world: 'shoes', equipped: 'shoes' },
};

function loadContract() {
  if (existsSync(CORE_ITEMS_PATH)) return JSON.parse(readFileSync(CORE_ITEMS_PATH, 'utf8'));
  return { source: 'fallback', items: Object.keys(VISUAL_HINTS).map((id) => ({ id, visual: {
    icon: `assets/items/${id}/icon.png`,
    world: `assets/items/${id}/world.png`,
    equipped: `assets/items/${id}/equipped.png`,
  }})) };
}

function writePng(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

function draw67Mark(c, x, y, fg, accent) {
  // Stylized "67" pixels.
  c.rect(x, y, 2, 6, fg); c.rect(x + 1, y, 3, 1, fg); c.rect(x + 3, y + 1, 1, 2, fg); c.rect(x + 1, y + 3, 2, 1, accent);
  c.rect(x + 6, y, 4, 1, fg); c.rect(x + 9, y + 1, 1, 4, fg); c.rect(x + 6, y + 5, 4, 1, fg); c.rect(x + 6, y + 3, 3, 1, accent);
}

function icon(bg, fg, accent = null) {
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, bg);
  c.rect(2, 2, 28, 2, '#ffffff33');
  c.rect(10, 8, 12, 16, fg);
  c.rect(12, 10, 8, 12, bg);
  c.rect(15, 6, 2, 20, '#ffffff55');
  if (accent) draw67Mark(c, 11, 22, fg, accent);
  return c;
}

function worldSprite(kind, fg, accent = null) {
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
  } else {
    c.rect(6, 6, 8, 8, fg);
  }
  if (accent && (kind === 'cap' || kind === 'shirt' || kind === 'shoes')) draw67Mark(c, 5, 1, fg, accent);
  return c;
}

function equippedSprite(kind, fg, accent = null) {
  // Layer sprite sized for composition; placeholders are rig-compatible enough for tooling.
  if (kind === 'blade' || kind === 'blade67' || kind === 'bow' || kind === 'gun') {
    const c = new Canvas(32, 32);
    c.rect(18, 8, 3, 16, fg);
    if (kind === 'blade67' && accent) c.rect(17, 11, 5, 2, accent);
    if (kind === 'gun') { c.rect(16, 12, 10, 5, P.crate); c.rect(24, 11, 4, 4, P.crateDark); }
    if (kind === 'bow') { c.rect(18, 8, 2, 16, P.wood); c.rect(20, 10, 3, 2, P.woodLight); c.rect(20, 20, 3, 2, P.woodLight); }
    return c;
  }
  if (kind === 'cap') {
    const c = new Canvas(32, 32);
    c.rect(9, 6, 14, 5, fg); c.rect(7, 10, 18, 4, fg);
    if (accent) c.rect(20, 11, 6, 2, accent);
    return c;
  }
  if (kind === 'shirt' || kind === 'plate') {
    const c = new Canvas(32, 32);
    c.rect(10, 12, 12, 12, fg); c.rect(7, 13, 3, 8, fg); c.rect(22, 13, 3, 8, fg);
    if (accent) c.rect(13, 15, 6, 4, accent);
    return c;
  }
  if (kind === 'shoes') {
    const c = new Canvas(32, 32);
    c.rect(9, 24, 6, 4, fg); c.rect(17, 24, 6, 4, fg);
    if (accent) { c.rect(10, 25, 4, 2, accent); c.rect(18, 25, 4, 2, accent); }
    return c;
  }
  return worldSprite(kind, fg, accent);
}

export function buildItemAssets({ force = false } = {}) {
  const contract = loadContract();
  const files = [];
  for (const item of contract.items || []) {
    const hint = VISUAL_HINTS[item.id] || { bg: '#444', fg: '#ddd', world: 'box', equipped: 'box' };
    const visual = item.visual || {};
    const assets = [
      ['icon', icon(hint.bg, hint.fg, hint.accent || null)],
      ['world', worldSprite(hint.world, hint.fg, hint.accent || null)],
      ['equipped', equippedSprite(hint.equipped || hint.world, hint.fg, hint.accent || null)],
    ];
    const dir = join(ROOT, 'assets/items', item.id);
    mkdirSync(dir, { recursive: true });
    for (const [slot, canvas] of assets) {
      const rel = visual[slot] || `assets/items/${item.id}/${slot}.png`;
      const abs = join(ROOT, rel);
      if (force || !existsSync(abs)) writePng(abs, canvas);
      files.push(rel);
    }
    writeFileSync(join(dir, 'manifest.json'), JSON.stringify({
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
      },
    }, null, 2) + '\n');
  }
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
