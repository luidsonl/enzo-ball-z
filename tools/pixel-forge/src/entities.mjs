import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

function heroine(c, ox, step, jump = false, hurt = false) {
  const skin = hurt ? '#ffd0d0' : '#f2c9a0';
  const shirt = hurt ? '#ff8a8a' : '#ff6fae';
  const pants = '#5a3f8f';
  const hair = '#5a3218';
  const y = jump ? -2 : 0;
  c.rect(ox + 11, 6 + y, 10, 8, skin); // head
  c.rect(ox + 10, 5 + y, 12, 4, hair); // hair top
  c.rect(ox + 9, 8 + y, 2, 7, hair); c.rect(ox + 21, 8 + y, 2, 7, hair); // hair sides
  c.rect(ox + 13, 9 + y, 2, 2, P.outline); c.rect(ox + 17, 9 + y, 2, 2, P.outline);
  c.rect(ox + 15, 11 + y, 2, 1, '#d98b8b');
  c.rect(ox + 10, 14 + y, 12, 9, shirt);
  c.rect(ox + 8, 15 + y, 2, 7, skin); c.rect(ox + 22, 15 + y, 2, 7, skin);
  c.rect(ox + 11, 23, 4, 7 + (step === 1 ? 1 : 0), pants);
  c.rect(ox + 17, 23, 4, 7 + (step === -1 ? 1 : 0), pants);
  c.rect(ox + 10, 29, 5, 2, '#3a2a1a'); c.rect(ox + 17, 29, 5, 2, '#3a2a1a');
}

const ENTITY_VISUALS = [
  {
    id: 'enzo', cell: 32, frames: [
      { name: 'idle', draw: (c, x) => hero(c, x, 0) },
      { name: 'walk_1', draw: (c, x) => hero(c, x, 1) },
      { name: 'walk_2', draw: (c, x) => hero(c, x, -1) },
      { name: 'jump', draw: (c, x) => hero(c, x, 0, true) },
      { name: 'hurt', draw: (c, x) => hero(c, x, 0, false, true) },
    ],
  },
  {
    id: 'valentina', cell: 32, frames: [
      { name: 'idle', draw: (c, x) => heroine(c, x, 0) },
      { name: 'walk_1', draw: (c, x) => heroine(c, x, 1) },
      { name: 'walk_2', draw: (c, x) => heroine(c, x, -1) },
      { name: 'jump', draw: (c, x) => heroine(c, x, 0, true) },
      { name: 'kick', draw: (c, x) => { heroine(c, x, 0); c.rect(x + 23, 24, 6, 2, '#3a2a1a'); } },
      { name: 'cast', draw: (c, x) => { heroine(c, x, 0); c.rect(x + 22, 16, 5, 3, '#5ef2ff'); } },
      { name: 'sixseven_a', draw: (c, x) => { heroine(c, x, 0); c.rect(x + 6, 10, 2, 8, '#f2c9a0'); c.rect(x + 24, 18, 2, 8, '#f2c9a0'); } },
      { name: 'sixseven_b', draw: (c, x) => { heroine(c, x, 0); c.rect(x + 6, 18, 2, 8, '#f2c9a0'); c.rect(x + 24, 10, 2, 8, '#f2c9a0'); } },
      { name: 'hurt', draw: (c, x) => heroine(c, x, 0, false, true) },
    ],
  },
  {
    id: 'slime', cell: 32, frames: [
      { name: 'walk_a', draw: (c, x) => slime(c, x, 0) },
      { name: 'walk_b', draw: (c, x) => slime(c, x, 1) },
      { name: 'stretch', draw: (c, x) => slime(c, x, 2) },
      { name: 'hurt', draw: (c, x) => slime(c, x, 3) },
    ],
  },
];

function writePng(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

function hero(c, ox, step, jump = false, hurt = false) {
  const skin = hurt ? '#ffd0d0' : '#f2c9a0';
  const shirt = hurt ? '#ff8a8a' : '#4f7cff';
  const pants = '#2f3a66';
  const y = jump ? -2 : 0;
  c.rect(ox + 11, 6 + y, 10, 8, skin); // head
  c.rect(ox + 11, 6 + y, 10, 3, '#5a3a22'); // hair
  c.rect(ox + 13, 9 + y, 2, 2, P.outline); c.rect(ox + 17, 9 + y, 2, 2, P.outline);
  c.rect(ox + 10, 14 + y, 12, 10, shirt);
  c.rect(ox + 8, 15 + y, 2, 8, skin); c.rect(ox + 22, 15 + y, 2, 8, skin);
  c.rect(ox + 11, 24, 4, 6 + (step === 1 ? 1 : 0), pants);
  c.rect(ox + 17, 24, 4, 6 + (step === -1 ? 1 : 0), pants);
  c.rect(ox + 10, 29, 5, 2, '#3a2a1a'); c.rect(ox + 17, 29, 5, 2, '#3a2a1a');
}

function slime(c, ox, frame) {
  const body = frame === 3 ? '#ff9aa8' : '#66d17a';
  const dark = frame === 3 ? '#d85a6a' : '#3f9e57';
  const h = frame === 2 ? 12 : frame === 1 ? 14 : 15;
  const w = frame === 2 ? 22 : 18;
  const x = ox + (32 - w) / 2;
  const y = 30 - h;
  c.rect(x, y, w, h, body);
  c.rect(x, y + h - 3, w, 3, dark);
  c.rect(x + w * 0.25, y + h * 0.35, 3, 4, P.outline);
  c.rect(x + w * 0.6, y + h * 0.35, 3, 4, P.outline);
  c.rect(x + w * 0.35, y + h * 0.65, w * 0.3, 2, '#ffffff66');
}

export function buildEntityAssets({ force = false } = {}) {
  const files = [];
  for (const entity of ENTITY_VISUALS) {
    const dir = join(ROOT, 'assets/entities', entity.id);
    const sheetPath = join(dir, 'sheet.png');
    const atlasPath = join(dir, 'atlas.json');
    const manifestPath = join(dir, 'manifest.json');
    if (force || !existsSync(sheetPath)) {
      const sheet = new Canvas(entity.cell * entity.frames.length, entity.cell);
      entity.frames.forEach((f, i) => f.draw(sheet, i * entity.cell));
      writePng(sheetPath, sheet);
    }
    const clipMap = {
      enzo: { idle: ['idle'], walk: ['walk_1', 'walk_2'], jump: ['jump'], hurt: ['hurt'] },
      valentina: { idle: ['idle'], walk: ['walk_1', 'walk_2'], jump: ['jump'], kick: ['kick'], cast: ['cast'], sixseven_a: ['sixseven_a'], sixseven_b: ['sixseven_b'], hurt: ['hurt'] },
      slime: { walk: ['walk_a', 'walk_b'], stretch: ['stretch'], hurt: ['hurt'] },
    };
    const atlas = {
      image: 'sheet.png', cell: { w: entity.cell, h: entity.cell },
      frames: Object.fromEntries(entity.frames.map((f, i) => [f.name, { x: i * entity.cell, y: 0, w: entity.cell, h: entity.cell }])),
      clips: clipMap[entity.id] || {},
    };
    writeFileSync(atlasPath, JSON.stringify(atlas, null, 2) + '\n');
    writeFileSync(manifestPath, JSON.stringify({ framework: 'pixel-forge', id: entity.id, sheet: 'sheet.png', atlas: 'atlas.json', clips: Object.keys(atlas.clips) }, null, 2) + '\n');
    files.push(`assets/entities/${entity.id}/sheet.png`, `assets/entities/${entity.id}/atlas.json`);
  }
  return files;
}

export function validateEntityAssets() {
  const errors = [];
  for (const entity of ENTITY_VISUALS) {
    for (const name of ['sheet.png', 'atlas.json', 'manifest.json']) {
      const p = join(ROOT, 'assets/entities', entity.id, name);
      if (!existsSync(p)) errors.push(`entity ${entity.id}: missing ${name}`);
    }
    const sheet = join(ROOT, 'assets/entities', entity.id, 'sheet.png');
    if (existsSync(sheet)) {
      const b = readFileSync(sheet);
      if (b[0] !== 137 || b[1] !== 80) errors.push(`entity ${entity.id}: invalid PNG sheet.png`);
    }
  }
  return errors;
}
