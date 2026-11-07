import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

const ITEM_VISUALS = [
  { id: 'sword', bg: '#3a4a6b', fg: '#d8e8ff', world: 'blade' },
  { id: 'bow', bg: '#5a3a22', fg: P.woodLight, world: 'bow' },
  { id: 'armor', bg: '#555b66', fg: '#cfd8e6', world: 'plate' },
  { id: 'amulet', bg: '#2f5a4a', fg: '#8fffd8', world: 'gem' },
  { id: 'potion', bg: '#6b1f3a', fg: '#ff8fb8', world: 'bottle' },
  { id: 'nutella_gun', bg: P.crateDark, fg: P.woodLight, world: 'gun' },
];

function writePng(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

function icon(bg, fg) {
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, bg);
  c.rect(2, 2, 28, 2, '#ffffff33');
  c.rect(10, 8, 12, 16, fg);
  c.rect(12, 10, 8, 12, bg);
  c.rect(15, 6, 2, 20, '#ffffff55');
  return c;
}

function worldSprite(kind, fg) {
  const c = new Canvas(20, 20);
  if (kind === 'blade') { c.rect(9,2,2,14,fg); c.rect(6,14,8,2,P.wood); c.rect(9,16,2,2,P.woodLight); }
  else if (kind === 'bow') { c.rect(8,3,2,14,P.wood); c.rect(10,4,3,2,P.woodLight); c.rect(10,14,3,2,P.woodLight); c.rect(12,6,1,8,'#ffffff66'); }
  else if (kind === 'plate') { c.rect(5,4,10,12,fg); c.rect(6,5,8,10,'#8a95a8'); c.rect(9,6,2,8,'#ffffff44'); }
  else if (kind === 'gem') { c.rect(9,4,2,2,fg); c.rect(8,6,4,6,fg); c.rect(9,12,2,3,P.gold); c.rect(9,7,2,4,'#ffffff55'); }
  else if (kind === 'bottle') { c.rect(8,7,4,9,fg); c.rect(9,4,2,3,'#d8f0ff'); c.rect(8,6,4,2,P.outline); c.rect(9,9,2,5,'#ffffff44'); }
  else { c.rect(5,9,10,4,P.crate); c.rect(13,8,3,3,P.crateDark); c.rect(7,7,3,2,P.wood); c.rect(8,10,5,2,'#ffffff33'); }
  return c;
}

export function buildItemAssets({ force = false } = {}) {
  const files = [];
  for (const item of ITEM_VISUALS) {
    const dir = join(ROOT, 'assets/items', item.id);
    const iconPath = join(dir, 'icon.png');
    const worldPath = join(dir, 'world.png');
    if (force || !existsSync(iconPath)) writePng(iconPath, icon(item.bg, item.fg));
    if (force || !existsSync(worldPath)) writePng(worldPath, worldSprite(item.world, item.fg));
    files.push(`assets/items/${item.id}/icon.png`, `assets/items/${item.id}/world.png`);
    writeFileSync(join(dir, 'manifest.json'), JSON.stringify({
      framework: 'pixel-forge',
      id: item.id,
      assets: { icon: 'icon.png', world: 'world.png' },
    }, null, 2) + '\n');
  }
  return files;
}

export function validateItemAssets() {
  const errors = [];
  for (const item of ITEM_VISUALS) {
    for (const name of ['icon.png', 'world.png']) {
      const p = join(ROOT, 'assets/items', item.id, name);
      if (!existsSync(p)) errors.push(`item ${item.id}: missing ${name}`);
      else {
        const b = readFileSync(p);
        if (b[0] !== 137 || b[1] !== 80) errors.push(`item ${item.id}: invalid PNG ${name}`);
      }
    }
  }
  return errors;
}
