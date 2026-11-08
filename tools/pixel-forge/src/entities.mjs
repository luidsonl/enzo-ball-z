import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

// Canonical humanoid rig: all states/clips have frame lists with explicit fallbacks.
// Every layer sheet uses the same frame order/positions so overlays align.
export const HUMANOID_FRAMES = [
  'idle_0', 'idle_1',
  'walk_0', 'walk_1', 'walk_2', 'walk_3',
  'run_0', 'run_1', 'run_2', 'run_3',
  'crouch_0',
  'jump_rise_0',
  'jump_fall_0',
  'fly_0', 'fly_1',
  'swim_0', 'swim_1',
  'dash_0',
  'sixseven_a_0',
  'sixseven_b_0',
  'attack_1_0',
  'attack_2_0',
  'attack_3_0',
  'kick_0',
  'cast_0',
  'hurt_0',
  'dying_0',
  'dead_0',
];

export const HUMANOID_CLIPS = {
  idle: { frames: ['idle_0', 'idle_1'], fps: 2.5, loop: true },
  walk: { frames: ['walk_0', 'walk_1', 'walk_2', 'walk_3'], fps: 8, loop: true },
  run: { frames: ['run_0', 'run_1', 'run_2', 'run_3'], fps: 10, loop: true },
  crouch: { frames: ['crouch_0'], fps: 1, loop: true },
  jump: { frames: ['jump_rise_0'], fps: 1, loop: true },
  jump_rise: { frames: ['jump_rise_0'], fps: 1, loop: true },
  jump_fall: { frames: ['jump_fall_0'], fps: 1, loop: true },
  fly: { frames: ['fly_0', 'fly_1'], fps: 6, loop: true },
  swim: { frames: ['swim_0', 'swim_1'], fps: 5, loop: true },
  dash: { frames: ['dash_0'], fps: 1, loop: true },
  sixseven_a: { frames: ['sixseven_a_0'], fps: 1, loop: true },
  sixseven_b: { frames: ['sixseven_b_0'], fps: 1, loop: true },
  attack_1: { frames: ['attack_1_0'], fps: 10, loop: false },
  attack_2: { frames: ['attack_2_0'], fps: 10, loop: false },
  attack_3: { frames: ['attack_3_0'], fps: 10, loop: false },
  kick: { frames: ['kick_0'], fps: 10, loop: false },
  cast: { frames: ['cast_0'], fps: 8, loop: false },
  hurt: { frames: ['hurt_0'], fps: 1, loop: true },
  dying: { frames: ['dying_0'], fps: 4, loop: false },
  dead: { frames: ['dead_0'], fps: 1, loop: true },
};

export const HUMANOID_FALLBACKS = {
  run: ['walk', 'idle'],
  crouch: ['idle'],
  fly: ['jump_rise', 'jump_fall', 'jump', 'idle'],
  swim: ['fly', 'walk', 'idle'],
  dash: ['run', 'walk', 'idle'],
  jump: ['jump_rise', 'jump_fall', 'idle'],
  attack_2: ['attack_1'],
  attack_3: ['attack_2', 'attack_1'],
  kick: ['attack_1'],
  cast: ['attack_1', 'idle'],
  dying: ['dead', 'hurt'],
  sixseven_a: ['idle'],
  sixseven_b: ['idle'],
};

function writePng(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

function hexToRgba(hex, a = 255) {
  if (!hex) return [0, 0, 0, 0];
  const h = hex.replace('#', '');
  const full = h.length === 8 ? h : h.length === 6 ? h + 'ff' : h.length === 3 ? h.split('').map(x=>x+x).join('') + 'ff' : '000000ff';
  return [parseInt(full.slice(0,2),16), parseInt(full.slice(2,4),16), parseInt(full.slice(4,6),16), Math.round((parseInt(full.slice(6,8),16)/255)*a)];
}

function poseFromFrame(frame) {
  if (frame.startsWith('walk_')) return { step: ['walk_1','walk_3'].includes(frame) ? 1 : (frame==='walk_2'? -1 : 0), air: 0, crouch: false, run: false, swim: false, fly: false };
  if (frame.startsWith('run_')) return { step: ['run_1','run_3'].includes(frame) ? 2 : (frame==='run_2'? -2 : 0), air: 0, crouch: false, run: true, swim: false, fly: false };
  if (frame.startsWith('idle_')) return { step: 0, air: frame==='idle_1'? -1 : 0, crouch: false, run: false, swim: false, fly: false };
  if (frame === 'crouch_0') return { step: 0, air: 0, crouch: true, run: false, swim: false, fly: false };
  if (frame === 'jump_rise_0') return { step: 0, air: -3, crouch: false, run: false, swim: false, fly: false };
  if (frame === 'jump_fall_0' || frame === 'dash_0') return { step: 1, air: -1, crouch: false, run: frame==='dash_0', swim: false, fly: false };
  if (frame.startsWith('fly_')) return { step: frame==='fly_1'?1:0, air: -2, crouch: false, run: false, swim: false, fly: true };
  if (frame.startsWith('swim_')) return { step: frame==='swim_1'?1:-1, air: -1, crouch: false, run: false, swim: true, fly: false };
  if (frame === 'dying_0' || frame === 'dead_0') return { step: 0, air: 0, crouch: true, run: false, swim: false, fly: false, dead: true };
  return { step: 0, air: 0, crouch: false, run: false, swim: false, fly: false };
}

function drawHumanoidLayer(c, ox, frame, charId, layer, profile = {}) {
  const pose = poseFromFrame(frame);
  const hurt = frame === 'hurt_0';
  const dead = frame === 'dying_0' || frame === 'dead_0';
  const skinBase = charId === 'valentina' ? '#f2c9a0' : '#f2c9a0';
  const skin = dead ? '#d8d0e8' : hurt ? '#ffd0d0' : skinBase;
  const shirtBase = charId === 'valentina' ? '#ff6fae' : '#4f7cff';
  const pantsBase = charId === 'valentina' ? '#5a3f8f' : '#2f3a66';
  const hairBase = charId === 'valentina' ? '#5a3218' : '#5a3a22';
  const shirt = hurt ? '#ff8a8a' : shirtBase;
  const pants = pantsBase;
  const hair = hairBase;
  const shoe = '#3a2a1a';
  const y = pose.air + (pose.crouch ? 2 : 0);
  const headY = 6 + y;
  const bodyY = 14 + y;
  const legH = pose.crouch ? 4 : (dead ? 3 : 6);
  const step = pose.step;
  const draw = (cond, fn) => { if (cond) fn(); };

  // base/skin: head, neck, arms, hands, legs skin
  draw(layer === 'full' || layer === 'base' || layer === 'skin', () => {
    c.rect(ox + 11, headY, 10, 8, skin); // head
    c.rect(ox + 8, bodyY + 1, 2, pose.crouch ? 4 : 7, skin); // left arm
    c.rect(ox + 22, bodyY + 1, 2, pose.crouch ? 4 : 7, skin); // right arm
    c.rect(ox + 11, 24 + (pose.crouch ? 2 : 0), 4, 2, skin); // leg tops visible
    c.rect(ox + 17, 24 + (pose.crouch ? 2 : 0), 4, 2, skin);
  });
  draw(layer === 'full' || layer === 'face', () => {
    c.rect(ox + 13, headY + 3, 2, 2, P.outline);
    c.rect(ox + 17, headY + 3, 2, 2, P.outline);
    c.rect(ox + 15, headY + 5, 2, 1, dead ? '#7a6a8a' : '#d98b8b');
  });
  draw(layer === 'full' || layer === 'hair', () => {
    if (charId === 'valentina') {
      c.rect(ox + 10, headY - 1, 12, 4, hair);
      c.rect(ox + 9, headY + 2, 2, 7, hair); c.rect(ox + 21, headY + 2, 2, 7, hair);
    } else {
      c.rect(ox + 11, headY, 10, 3, hair);
    }
  });
  draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
    c.rect(ox + 10, bodyY, 12, pose.crouch ? 7 : 10, shirt);
  });
  draw(layer === 'full' || layer === 'outfit_bottom' || layer === 'pants', () => {
    c.rect(ox + 11, 23 + (pose.crouch ? 2 : 0), 4, legH + (step === 1 || step === 2 ? 1 : 0), pants);
    c.rect(ox + 17, 23 + (pose.crouch ? 2 : 0), 4, legH + (step === -1 || step === -2 ? 1 : 0), pants);
  });
  draw(layer === 'full' || layer === 'shoes', () => {
    const fy = 29;
    if (!dead) {
      c.rect(ox + 10, fy, 5, 2, shoe); c.rect(ox + 17, fy, 5, 2, shoe);
    } else {
      c.rect(ox + 8, fy - 1, 8, 2, shoe); c.rect(ox + 16, fy - 1, 8, 2, shoe);
    }
  });

  // action overlays baked into full fallback for readability; layer system can add/override
  if (layer === 'full' || layer === 'action_overlay') {
    if (frame === 'sixseven_a_0') { c.rect(ox + 6, bodyY - 2, 2, 8, skin); c.rect(ox + 24, bodyY + 4, 2, 8, skin); }
    if (frame === 'sixseven_b_0') { c.rect(ox + 6, bodyY + 4, 2, 8, skin); c.rect(ox + 24, bodyY - 2, 2, 8, skin); }
    if (frame === 'attack_1_0') c.rect(ox + 23, bodyY + 2, 6, 3, skin);
    if (frame === 'attack_2_0') c.rect(ox + 23, bodyY + 4, 7, 3, skin);
    if (frame === 'attack_3_0') c.rect(ox + 22, bodyY + 1, 8, 4, skin);
    if (frame === 'kick_0') c.rect(ox + 22, 24, 7, 2, shoe);
    if (frame === 'cast_0') c.rect(ox + 22, bodyY + 1, 6, 4, '#5ef2ff88');
    if (frame === 'fly_0' || frame === 'fly_1') c.rect(ox + 12, 27, 8, 2, '#5ef2ff55');
    if (frame === 'swim_0' || frame === 'swim_1') c.rect(ox + 6, 26, 20, 3, '#4f7cff44');
    if (frame === 'dash_0') c.rect(ox + 3, bodyY + 4, 6, 8, '#ffffff22');
  }
}

function drawSlime(c, ox, frame) {
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

function makeRig(characterId, cell = 32) {
  const frames = Object.fromEntries(HUMANOID_FRAMES.map((name, i) => [name, { x: i * cell, y: 0, w: cell, h: cell }]));
  return {
    framework: 'pixel-forge',
    id: characterId,
    kind: 'humanoid',
    image: 'sheet.png',
    cell: { w: cell, h: cell },
    anchor: { x: Math.floor(cell / 2), y: cell - 2 },
    frames,
    clips: HUMANOID_CLIPS,
    fallbacks: HUMANOID_FALLBACKS,
    layers: ['base', 'face', 'hair', 'outfit_bottom', 'shoes', 'outfit_top', 'headgear', 'accessory', 'weapon'],
  };
}

function buildHumanoid(id, cell = 32) {
  const dir = join(ROOT, 'assets/entities', id);
  mkdirSync(join(dir, 'layers'), { recursive: true });
  const sheet = new Canvas(cell * HUMANOID_FRAMES.length, cell);
  HUMANOID_FRAMES.forEach((frame, i) => drawHumanoidLayer(sheet, i * cell, frame, id, 'full'));
  writePng(join(dir, 'sheet.png'), sheet);

  for (const layer of ['base', 'face', 'hair', 'outfit_top', 'outfit_bottom', 'shoes']) {
    const c = new Canvas(cell * HUMANOID_FRAMES.length, cell);
    HUMANOID_FRAMES.forEach((frame, i) => drawHumanoidLayer(c, i * cell, frame, id, layer));
    writePng(join(dir, 'layers', `${layer}.png`), c);
  }

  const rig = makeRig(id, cell);
  writeFileSync(join(dir, 'rig.json'), JSON.stringify(rig, null, 2) + '\n');
  writeFileSync(join(dir, 'atlas.json'), JSON.stringify({
    image: 'sheet.png', cell: rig.cell, anchor: rig.anchor, frames: rig.frames, clips: rig.clips, fallbacks: rig.fallbacks,
  }, null, 2) + '\n');
  writeFileSync(join(dir, 'manifest.json'), JSON.stringify({
    framework: 'pixel-forge', id, kind: 'humanoid', sheet: 'sheet.png', rig: 'rig.json', atlas: 'atlas.json',
    clips: Object.keys(rig.clips), layers: rig.layers,
  }, null, 2) + '\n');
  return [`assets/entities/${id}/sheet.png`, `assets/entities/${id}/rig.json`, `assets/entities/${id}/atlas.json`];
}

export function buildEntityAssets({ force = false } = {}) {
  const files = [];
  // Humanoids always regenerate rig-aligned assets (deterministic, contract-driven).
  files.push(...buildHumanoid('enzo'));
  files.push(...buildHumanoid('valentina'));

  const slimeDir = join(ROOT, 'assets/entities/slime');
  const slimeFrames = [
    { name: 'walk_a', draw: (c, x) => drawSlime(c, x, 'walk_a') },
    { name: 'walk_b', draw: (c, x) => drawSlime(c, x, 'walk_b') },
    { name: 'stretch', draw: (c, x) => drawSlime(c, x, 'stretch') },
    { name: 'hurt', draw: (c, x) => drawSlime(c, x, 'hurt') },
  ];
  const slimeSheet = new Canvas(32 * slimeFrames.length, 32);
  slimeFrames.forEach((f, i) => f.draw(slimeSheet, i * 32));
  mkdirSync(slimeDir, { recursive: true });
  writePng(join(slimeDir, 'sheet.png'), slimeSheet);
  const slimeAtlas = {
    image: 'sheet.png', cell: { w: 32, h: 32 },
    frames: Object.fromEntries(slimeFrames.map((f, i) => [f.name, { x: i * 32, y: 0, w: 32, h: 32 }])),
    clips: { walk: ['walk_a', 'walk_b'], stretch: ['stretch'], hurt: ['hurt'] },
  };
  writeFileSync(join(slimeDir, 'atlas.json'), JSON.stringify(slimeAtlas, null, 2) + '\n');
  writeFileSync(join(slimeDir, 'manifest.json'), JSON.stringify({ framework: 'pixel-forge', id: 'slime', sheet: 'sheet.png', atlas: 'atlas.json', clips: Object.keys(slimeAtlas.clips) }, null, 2) + '\n');
  files.push('assets/entities/slime/sheet.png', 'assets/entities/slime/atlas.json');
  return files;
}

export function validateEntityAssets() {
  const errors = [];
  for (const entity of ['enzo', 'valentina']) {
    for (const name of ['sheet.png', 'atlas.json', 'manifest.json', 'rig.json']) {
      const p = join(ROOT, 'assets/entities', entity, name);
      if (!existsSync(p)) errors.push(`entity ${entity}: missing ${name}`);
    }
    const rigPath = join(ROOT, 'assets/entities', entity, 'rig.json');
    if (existsSync(rigPath)) {
      try {
        const rig = JSON.parse(readFileSync(rigPath, 'utf8'));
        for (const clip of ['idle', 'walk', 'jump_rise', 'attack_1', 'sixseven_a', 'hurt', 'dead']) {
          if (!rig.clips?.[clip]) errors.push(`entity ${entity}: rig missing clip ${clip}`);
        }
        if (!Array.isArray(rig.layers)) errors.push(`entity ${entity}: rig missing layers`);
      } catch (e) {
        errors.push(`entity ${entity}: invalid rig.json (${e.message})`);
      }
    }
    for (const layer of ['base', 'face', 'hair', 'outfit_top', 'outfit_bottom', 'shoes']) {
      const p = join(ROOT, 'assets/entities', entity, 'layers', `${layer}.png`);
      if (!existsSync(p)) errors.push(`entity ${entity}: missing layer ${layer}.png`);
    }
  }
  for (const name of ['sheet.png', 'atlas.json', 'manifest.json']) {
    const p = join(ROOT, 'assets/entities/slime', name);
    if (!existsSync(p)) errors.push(`entity slime: missing ${name}`);
  }
  for (const id of ['enzo', 'valentina', 'slime']) {
    const sheet = join(ROOT, 'assets/entities', id, 'sheet.png');
    if (existsSync(sheet)) {
      const b = readFileSync(sheet);
      if (b[0] !== 137 || b[1] !== 80) errors.push(`entity ${id}: invalid PNG sheet.png`);
    }
  }
  return errors;
}
