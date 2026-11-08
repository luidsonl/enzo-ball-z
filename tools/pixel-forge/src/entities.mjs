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
  'climb_0', 'climb_1',
  'dash_0',
  'sixseven_a_0',
  'sixseven_b_0',
  'attack_1_0',
  'attack_2_0',
  'attack_3_0',
  'kick_0',
  'cast_0',
  'shout_0',
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
  climb: { frames: ['climb_0', 'climb_1'], fps: 5, loop: true },
  dash: { frames: ['dash_0'], fps: 1, loop: true },
  sixseven_a: { frames: ['sixseven_a_0'], fps: 1, loop: true },
  sixseven_b: { frames: ['sixseven_b_0'], fps: 1, loop: true },
  attack_1: { frames: ['attack_1_0'], fps: 10, loop: false },
  attack_2: { frames: ['attack_2_0'], fps: 10, loop: false },
  attack_3: { frames: ['attack_3_0'], fps: 10, loop: false },
  kick: { frames: ['kick_0'], fps: 10, loop: false },
  cast: { frames: ['cast_0'], fps: 8, loop: false },
  shout: { frames: ['shout_0'], fps: 8, loop: false },
  hurt: { frames: ['hurt_0'], fps: 1, loop: true },
  dying: { frames: ['dying_0'], fps: 4, loop: false },
  dead: { frames: ['dead_0'], fps: 1, loop: true },
};

export const HUMANOID_FALLBACKS = {
  run: ['walk', 'idle'],
  crouch: ['idle'],
  fly: ['jump_rise', 'jump_fall', 'jump', 'idle'],
  swim: ['fly', 'walk', 'idle'],
  climb: ['jump_rise', 'idle'],
  dash: ['run', 'walk', 'idle'],
  jump: ['jump_rise', 'jump_fall', 'idle'],
  attack_2: ['attack_1'],
  attack_3: ['attack_2', 'attack_1'],
  kick: ['attack_1'],
  cast: ['attack_1', 'idle'],
  shout: ['cast', 'attack_1', 'idle'],
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
  const f = frame;
  // Rich pose data for expressive 32x32 humanoid animation (keeps 31-frame rig).
  const base = {
    step: 0, air: 0, crouch: false, run: false, swim: false, fly: false, climb: false, dead: false,
    bob: 0, lean: 0, armL: 0, armR: 0, legL: 0, legR: 0, armUpL: false, armUpR: false, armOutL: false, armOutR: false,
  };
  if (f.startsWith('idle_')) return { ...base, bob: f === 'idle_1' ? -1 : 0, armL: 1, armR: 1 };
  if (f === 'walk_0') return { ...base, bob: 0, legL: -1, legR: 1, armL: 2, armR: -1 };
  if (f === 'walk_1') return { ...base, step: 1, bob: -1, legL: 1, legR: -1, armL: -1, armR: 2 };
  if (f === 'walk_2') return { ...base, step: -1, bob: 0, legL: -1, legR: 1, armL: 2, armR: -1 };
  if (f === 'walk_3') return { ...base, step: 1, bob: -1, legL: 1, legR: -1, armL: -1, armR: 2 };
  if (f === 'run_0') return { ...base, run: true, bob: -1, lean: 1, legL: -2, legR: 2, armL: 3, armR: -2 };
  if (f === 'run_1') return { ...base, run: true, step: 2, bob: 0, lean: 1, legL: 2, legR: -2, armL: -2, armR: 3 };
  if (f === 'run_2') return { ...base, run: true, step: -2, bob: -1, lean: 1, legL: -2, legR: 2, armL: 3, armR: -2 };
  if (f === 'run_3') return { ...base, run: true, step: 2, bob: 0, lean: 1, legL: 2, legR: -2, armL: -2, armR: 3 };
  if (f === 'crouch_0') return { ...base, crouch: true, bob: 2, armL: 2, armR: 2, legL: 0, legR: 0 };
  if (f === 'jump_rise_0') return { ...base, air: -3, bob: -1, armUpL: true, armUpR: true, legL: -1, legR: 1 };
  if (f === 'jump_fall_0') return { ...base, air: -1, step: 1, bob: 1, armOutL: true, armOutR: true, legL: 1, legR: -1 };
  if (f === 'fly_0') return { ...base, fly: true, air: -2, bob: -1, armOutL: true, armOutR: true, legL: -1, legR: 1 };
  if (f === 'fly_1') return { ...base, fly: true, air: -2, bob: 0, armOutL: true, armOutR: true, legL: 1, legR: -1 };
  if (f === 'swim_0') return { ...base, swim: true, air: -1, bob: 1, lean: 1, armUpL: true, armR: 3, legL: -1, legR: 1 };
  if (f === 'swim_1') return { ...base, swim: true, air: -1, bob: 1, lean: 1, armL: 3, armUpR: true, legL: 1, legR: -1 };
  if (f === 'climb_0') return { ...base, climb: true, bob: 0, armUpL: true, armR: 2, legL: -1, legR: 1 };
  if (f === 'climb_1') return { ...base, climb: true, bob: -1, armL: 2, armUpR: true, legL: 1, legR: -1 };
  if (f === 'dash_0') return { ...base, run: true, step: 1, air: -1, lean: 2, bob: 0, armL: -1, armR: 4, legL: -2, legR: 2 };
  if (f === 'sixseven_a_0') return { ...base, bob: -1, armUpL: true, armR: 3, legL: 0, legR: 0 }; // aura farming: left up, right down
  if (f === 'sixseven_b_0') return { ...base, bob: -1, armL: 3, armUpR: true, legL: 0, legR: 0 }; // alternate
  if (f === 'attack_1_0') return { ...base, lean: 1, armL: 1, armOutR: true, legL: -1, legR: 1 };
  if (f === 'attack_2_0') return { ...base, lean: 2, armL: 2, armOutR: true, legL: -1, legR: 2 };
  if (f === 'attack_3_0') return { ...base, lean: 2, bob: -1, armUpL: true, armOutR: true, legL: -1, legR: 2 };
  if (f === 'kick_0') return { ...base, lean: 1, armL: 2, armR: 2, legL: -1, legR: -3 };
  if (f === 'cast_0') return { ...base, bob: -1, armL: 1, armOutR: true };
  if (f === 'shout_0') return { ...base, bob: -1, armOutL: true, armOutR: true, lean: 0 };
  if (f === 'hurt_0') return { ...base, bob: 1, lean: -1, armL: 2, armR: 2, legL: 1, legR: 0 };
  if (f === 'dying_0') return { ...base, crouch: true, dead: true, bob: 3, lean: -1, armL: 3, armR: 3, legL: 0, legR: 0 };
  if (f === 'dead_0') return { ...base, crouch: true, dead: true, bob: 4, lean: -2, armL: 4, armR: 4, legL: 0, legR: 0 };
  return base;
}

function drawHumanoidLayer(c, ox, frame, charId, layer, variant = 0, profile = {}) {
  const pose = poseFromFrame(frame);
  const hurt = frame === 'hurt_0';
  const dead = frame === 'dying_0' || frame === 'dead_0';
  const isVal = charId === 'valentina';
  const skinBase = '#f2c9a0';
  const skin = dead ? '#d8d0e8' : hurt ? '#ffd0d0' : skinBase;
  const shirtBase = isVal ? '#ff6fae' : '#4f7cff';
  const pantsBase = isVal ? '#5a3f8f' : '#2f3a66';
  const hairBase = isVal ? '#5a3218' : '#5a3a22';
  const shirt = hurt ? '#ff8a8a' : shirtBase;
  const pants = pantsBase;
  const hair = hairBase;
  const shoe = '#3a2a1a';
  const hairStyle = layer === 'hair' ? variant : (profile.hair_style ?? 0);
  const faceStyle = layer === 'face' ? variant : (profile.face ?? 0);
  const y = pose.air + (pose.crouch ? 2 : 0) + (pose.bob || 0);
  const lean = pose.lean || 0;
  const headY = 6 + y;
  const bodyY = 14 + y;
  const legH = pose.crouch ? 4 : (dead ? 3 : 6);
  const draw = (cond, fn) => { if (cond) fn(); };

  const armRect = (x, y0, up, out, dy) => {
    // expressive arm pose: up = raised overhead, out = extended to the side/forward, else down/side
    if (up) c.rect(ox + x, bodyY - 3 + dy, 2, 8, skin);
    else if (out) {
      if (x > 16) c.rect(ox + x, bodyY + 2 + dy, 7, 3, skin); // right arm extends right
      else c.rect(ox + x - 5, bodyY + 2 + dy, 7, 3, skin); // left arm extends left
    } else c.rect(ox + x, bodyY + 1 + dy, 2, pose.crouch ? 4 : 7, skin);
  };

  draw(layer === 'full' || layer === 'base' || layer === 'skin', () => {
    c.rect(ox + 11 + lean, headY, 10, 8, skin); // head
    c.rect(ox + 13 + lean, headY + 8, 6, 2, skin); // neck
    const ldy = pose.armL || 0;
    const rdy = pose.armR || 0;
    armRect(8 + lean, bodyY, !!pose.armUpL, !!pose.armOutL, ldy);
    armRect(22 + lean, bodyY, !!pose.armUpR, !!pose.armOutR, rdy);
    // legs (skin/base)
    const ly = 24 + (pose.crouch ? 2 : 0);
    const ll = Math.max(1, legH - 2 + (pose.legL || 0));
    const rl = Math.max(1, legH - 2 + (pose.legR || 0));
    if (pose.climb) {
      c.rect(ox + 11, 25, 4, pose.legL > 0 ? 3 : 2, skin);
      c.rect(ox + 17, 25, 4, pose.legR > 0 ? 3 : 2, skin);
    } else {
      c.rect(ox + 11, ly - (pose.legL < 0 ? 1 : 0), 4, ll, skin);
      c.rect(ox + 17, ly - (pose.legR < 0 ? 1 : 0), 4, rl, skin);
    }
  });
  draw(layer === 'full' || layer === 'face', () => {
    const hx = ox + lean;
    const eyeY = headY + 3;
    const eye = dead ? '#7a6a8a' : P.outline;
    const mouthY = headY + 6;
    const fs = Math.max(0, Math.min(7, faceStyle|0));
    if (isVal) {
      // Feminine face variants (Valentina exclusive)
      if (fs === 0) { c.rect(hx + 13, eyeY, 2, 2, '#3a2418'); c.rect(hx + 17, eyeY, 2, 2, '#3a2418'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); c.rect(hx + 12, eyeY + 2, 1, 1, '#e8a0b0'); c.rect(hx + 19, eyeY + 2, 1, 1, '#e8a0b0'); }
      else if (fs === 1) { c.rect(hx + 13, eyeY, 2, 2, '#5a3820'); c.rect(hx + 17, eyeY, 2, 2, '#5a3820'); c.rect(hx + 12, eyeY - 1, 3, 1, '#3a2418'); c.rect(hx + 17, eyeY - 1, 3, 1, '#3a2418'); }
      else if (fs === 2) { c.rect(hx + 13, eyeY, 2, 2, '#ffffff'); c.rect(hx + 17, eyeY, 2, 2, '#ffffff'); c.set(hx + 14, eyeY + 1, '#5a3820'); c.set(hx + 18, eyeY + 1, '#5a3820'); c.rect(hx + 12, eyeY + 2, 2, 1, '#f0b8c0'); c.rect(hx + 18, eyeY + 2, 2, 1, '#f0b8c0'); }
      else if (fs === 3) { c.rect(hx + 13, eyeY, 2, 1, eye); c.rect(hx + 17, eyeY, 2, 1, eye); c.set(hx + 13, eyeY + 1, eye); c.set(hx + 18, eyeY + 1, eye); }
      else if (fs === 4) { c.rect(hx + 13, eyeY, 2, 2, '#6b4226'); c.rect(hx + 17, eyeY, 2, 2, '#6b4226'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); }
      else if (fs === 5) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.set(hx + 19, eyeY + 2, '#f0b8c0'); }
      else if (fs === 6) { c.rect(hx + 12, eyeY, 3, 2, '#4a2818'); c.rect(hx + 17, eyeY, 3, 2, '#4a2818'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 19, eyeY, '#ffffff'); }
      else { c.rect(hx + 13, eyeY, 2, 2, '#7a4a28'); c.rect(hx + 17, eyeY, 2, 2, '#7a4a28'); c.rect(hx + 12, eyeY - 1, 1, 1, '#3a2418'); c.rect(hx + 19, eyeY - 1, 1, 1, '#3a2418'); }
      if (hurt) { c.rect(hx + 12, eyeY - 1, 3, 1, '#c06060'); c.rect(hx + 17, eyeY - 1, 3, 1, '#c06060'); }
      if (frame === 'shout_0') c.rect(hx + 14, mouthY - 1, 4, 3, '#4a2030');
      else if (dead) c.rect(hx + 15, mouthY, 2, 1, '#7a6a8a');
      else if (hurt) c.rect(hx + 14, mouthY, 3, 1, '#c06060');
      else if (fs === 0 || fs === 2 || fs === 4) { c.rect(hx + 14, mouthY, 4, 1, '#e07a8a'); c.set(hx + 14, mouthY - 1, '#e07a8a'); c.set(hx + 17, mouthY - 1, '#e07a8a'); }
      else c.rect(hx + 15, mouthY - 1, 2, 1, '#d98b8b');
    } else {
      // Masculine face variants (Enzo exclusive)
      if (fs === 0) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY, 2, 2, eye); c.rect(hx + 12, eyeY - 1, 3, 1, '#3a2418'); c.rect(hx + 17, eyeY - 1, 3, 1, '#3a2418'); }
      else if (fs === 1) { c.rect(hx + 13, eyeY + 1, 2, 1, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.rect(hx + 12, eyeY, 4, 1, '#4a3422'); c.rect(hx + 16, eyeY, 4, 1, '#4a3422'); }
      else if (fs === 2) { c.rect(hx + 12, eyeY, 3, 1, eye); c.rect(hx + 17, eyeY, 3, 1, eye); c.set(hx + 14, eyeY + 1, eye); c.set(hx + 18, eyeY + 1, eye); }
      else if (fs === 3) { c.rect(hx + 13, eyeY, 2, 2, '#5a3820'); c.rect(hx + 17, eyeY, 2, 2, '#5a3820'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); }
      else if (fs === 4) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY, 2, 2, eye); c.rect(hx + 14, mouthY + 1, 4, 1, '#8a6a4a'); } // stubble hint
      else if (fs === 5) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.rect(hx + 12, eyeY - 1, 2, 1, '#3a2418'); }
      else if (fs === 6) { c.rect(hx + 12, eyeY, 3, 2, eye); c.rect(hx + 17, eyeY, 3, 2, eye); c.rect(hx + 13, mouthY + 1, 6, 1, '#6a4a32'); } // beard hint
      else { c.rect(hx + 13, eyeY, 2, 1, eye); c.rect(hx + 17, eyeY, 2, 1, eye); c.rect(hx + 12, eyeY - 1, 8, 1, '#3a2418'); }
      if (hurt) { c.rect(hx + 12, eyeY - 1, 3, 1, '#c06060'); c.rect(hx + 17, eyeY - 1, 3, 1, '#c06060'); }
      if (frame === 'shout_0') c.rect(hx + 14, mouthY - 1, 4, 3, '#4a2030');
      else if (dead) c.rect(hx + 15, mouthY, 2, 1, '#7a6a8a');
      else if (hurt) c.rect(hx + 14, mouthY, 3, 1, '#c06060');
      else if (fs === 2) c.rect(hx + 14, mouthY, 4, 1, P.outline);
      else c.rect(hx + 15, mouthY - 1, 2, 1, '#b07060');
    }
  });
  draw(layer === 'full' || layer === 'hair', () => {
    const hx = ox + lean;
    const hs = Math.max(0, Math.min(7, hairStyle|0));
    if (isVal) {
      // Feminine hair variants (Valentina exclusive)
      if (hs === 0) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, 10, hair); c.rect(hx + 21, headY + 2, 2, 10, hair); } // long straight
      else if (hs === 1) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 15, headY - 2, 2, 2, hair); c.rect(hx + 21, headY + 2, 2, 8, hair); c.rect(hx + 22, headY + 8, 1, 2, hair); } // ponytail
      else if (hs === 2) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, 8, hair); c.rect(hx + 21, headY + 2, 2, 8, hair); } // long sides
      else if (hs === 3) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 1, 3, 6, hair); c.rect(hx + 20, headY + 1, 3, 6, hair); c.rect(hx + 10, headY + 6, 2, 2, hair); c.rect(hx + 20, headY + 6, 2, 2, hair); } // wavy
      else if (hs === 4) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, 5, hair); c.rect(hx + 21, headY + 2, 2, 5, hair); } // bob
      else if (hs === 5) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 9, headY + 2, 2, 7, hair); c.rect(hx + 21, headY + 2, 2, 7, hair); c.rect(hx + 9, headY + 8, 1, 2, hair); c.rect(hx + 22, headY + 8, 1, 2, hair); } // braided/pigtails hint
      else if (hs === 6) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 14, headY - 3, 4, 2, hair); c.rect(hx + 9, headY + 2, 2, 4, hair); c.rect(hx + 21, headY + 2, 2, 4, hair); } // bun/updo
      else { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 3, 9, hair); c.rect(hx + 20, headY + 2, 3, 9, hair); c.rect(hx + 12, headY + 10, 8, 1, hair); } // long back
    } else {
      // Masculine hair variants (Enzo exclusive). 0 = bald.
      if (hs === 0) { /* bald */ }
      else if (hs === 1) { c.rect(hx + 11, headY + 1, 10, 2, hair); c.rect(hx + 10, headY + 2, 1, 1, hair); c.rect(hx + 21, headY + 2, 1, 1, hair); } // short
      else if (hs === 2) { c.rect(hx + 11, headY + 1, 10, 1, hair); } // buzz
      else if (hs === 3) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 12, headY - 1, 2, 1, hair); c.rect(hx + 15, headY - 2, 2, 2, hair); c.rect(hx + 18, headY - 1, 2, 1, hair); } // spiky
      else if (hs === 4) { c.rect(hx + 10, headY, 11, 3, hair); c.rect(hx + 9, headY + 1, 2, 2, hair); c.rect(hx + 20, headY, 2, 1, hair); } // swept/side part
      else if (hs === 5) { c.rect(hx + 11, headY, 10, 2, hair); c.rect(hx + 10, headY + 1, 1, 2, hair); c.rect(hx + 21, headY + 1, 1, 2, hair); c.rect(hx + 14, headY - 1, 4, 1, hair); } // crew
      else if (hs === 6) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 1, 2, 3, hair); c.rect(hx + 21, headY + 1, 2, 3, hair); } // curly short
      else { c.rect(hx + 10, headY, 12, 3, hair); c.rect(hx + 9, headY + 2, 2, 3, hair); c.rect(hx + 21, headY + 2, 2, 2, hair); } // medium masculine
    }
  });
  draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
    const tx = ox + lean;
    c.rect(tx + 10, bodyY, 12, pose.crouch ? 7 : 10, shirt);
    c.rect(tx + 10, bodyY, 12, 1, hurt ? '#ffaaaa' : (isVal ? '#ff9fc8' : '#7fa0ff'));
    // sleeves hint (follow arm direction lightly)
    if (pose.armUpL) c.rect(tx + 8, bodyY - 1, 3, 3, shirt);
    else c.rect(tx + 8, bodyY + 1, 3, 2, shirt);
    if (pose.armUpR) c.rect(tx + 21, bodyY - 1, 3, 3, shirt);
    else if (pose.armOutR) c.rect(tx + 21, bodyY + 2, 3, 2, shirt);
    else c.rect(tx + 21, bodyY + 1, 3, 2, shirt);
  });
  draw(layer === 'full' || layer === 'outfit_bottom' || layer === 'pants', () => {
    const ly = 23 + (pose.crouch ? 2 : 0);
    if (pose.climb) {
      c.rect(ox + 11, 24, 4, frame === 'climb_1' ? 5 : 4, pants);
      c.rect(ox + 17, 24, 4, frame === 'climb_1' ? 4 : 5, pants);
    } else {
      c.rect(ox + 11, ly - (pose.legL < 0 ? 1 : 0), 4, legH + Math.max(0, pose.legL || 0), pants);
      c.rect(ox + 17, ly - (pose.legR < 0 ? 1 : 0), 4, legH + Math.max(0, pose.legR || 0), pants);
    }
  });
  draw(layer === 'full' || layer === 'shoes', () => {
    const fy = 29;
    if (!dead) {
      const ls = pose.legL < 0 ? -1 : 0;
      const rs = pose.legR < 0 ? -1 : 0;
      c.rect(ox + 10 + (frame === 'kick_0' ? 0 : 0), fy + ls, 5, 2, shoe);
      c.rect(ox + 17 + (frame === 'kick_0' ? 3 : 0), fy + rs + (frame === 'kick_0' ? -3 : 0), 6, 2, shoe);
      if (frame === 'kick_0') c.rect(ox + 22, fy - 4, 2, 3, pants); // extended kicking leg hint
    } else {
      c.rect(ox + 8, fy - 1, 8, 2, shoe); c.rect(ox + 16, fy - 1, 8, 2, shoe);
    }
  });

  if (layer === 'full' || layer === 'action_overlay') {
    // reinforce key actions for readability
    if (frame === 'sixseven_a_0') { c.rect(ox + 9, bodyY - 5, 3, 3, skin); c.rect(ox + 20, bodyY + 8, 3, 3, skin); }
    if (frame === 'sixseven_b_0') { c.rect(ox + 9, bodyY + 8, 3, 3, skin); c.rect(ox + 20, bodyY - 5, 3, 3, skin); }
    if (frame === 'attack_1_0') c.rect(ox + 24, bodyY + 2, 6, 3, skin);
    if (frame === 'attack_2_0') c.rect(ox + 24, bodyY + 3, 7, 3, skin);
    if (frame === 'attack_3_0') { c.rect(ox + 23, bodyY + 1, 8, 4, skin); c.rect(ox + 26, bodyY, 3, 2, '#ffffff55'); }
    if (frame === 'kick_0') c.rect(ox + 23, 25, 7, 2, shoe);
    if (frame === 'cast_0') { c.rect(ox + 24, bodyY + 1, 5, 3, skin); c.rect(ox + 27, bodyY, 3, 5, '#5ef2ff88'); }
    if (frame === 'shout_0') { c.rect(ox + 4, bodyY - 2, 4, 3, '#f6c44566'); c.rect(ox + 24, bodyY - 2, 4, 3, '#f6c44566'); c.rect(ox + 13, headY - 2, 6, 2, '#f6c44544'); }
    if (frame === 'fly_0' || frame === 'fly_1') c.rect(ox + 10, 28, 12, 2, frame === 'fly_1' ? '#5ef2ff77' : '#5ef2ff44');
    if (frame === 'swim_0' || frame === 'swim_1') c.rect(ox + 5, 27, 22, 2, '#4f7cff55');
    if (frame === 'hurt_0') c.rect(ox + 24, bodyY - 2, 4, 4, '#ff9aa844');
    if (frame === 'dash_0') c.rect(ox + 2, bodyY + 3, 7, 10, '#ffffff22');
  }
}

export const CUSTOMIZATION_VARIANTS = {
  skin_tone: 8,
  hair_style: 8,
  hair_color: 8,
  face: 8,
  outfit_top: 8,
  outfit_bottom: 8,
  shoes: 8,
};

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
    customization: {
      variants: CUSTOMIZATION_VARIANTS,
      layer_variants: {
        hair: Array.from({length: CUSTOMIZATION_VARIANTS.hair_style}, (_, i) => `layers/hair_style_${i}.png`),
        face: Array.from({length: CUSTOMIZATION_VARIANTS.face}, (_, i) => `layers/face_${i}.png`),
      },
    },
  };
}

function buildLayerStrip(id, layer, variant = 0, cell = 32) {
  const c = new Canvas(cell * HUMANOID_FRAMES.length, cell);
  HUMANOID_FRAMES.forEach((frame, i) => drawHumanoidLayer(c, i * cell, frame, id, layer, variant));
  return c;
}

function buildHumanoid(id, cell = 32) {
  const dir = join(ROOT, 'assets/entities', id);
  mkdirSync(join(dir, 'layers'), { recursive: true });
  // Gender-exclusive defaults: Enzo base is short hair (not bald), Valentina keeps feminine default.
  const defaultHair = id === 'valentina' ? 0 : 1;
  const defaultFace = 0;
  const sheet = buildLayerStrip(id, 'full', 0, cell); // full fallback; runtime uses rig layers/variants
  // For the base sheet fallback, render with defaults by drawing full then overriding is complex;
  // keep deterministic full(0) and rely on rig layer composition/variants in runtime.
  writePng(join(dir, 'sheet.png'), buildLayerStrip(id, 'full', 0, cell));

  for (const layer of ['base', 'outfit_top', 'outfit_bottom', 'shoes']) {
    const c = buildLayerStrip(id, layer, 0, cell);
    writePng(join(dir, 'layers', `${layer}.png`), c);
  }
  writePng(join(dir, 'layers', 'hair.png'), buildLayerStrip(id, 'hair', defaultHair, cell));
  writePng(join(dir, 'layers', 'face.png'), buildLayerStrip(id, 'face', defaultFace, cell));
  // Style/face variants as full rig-aligned strips (all animation states).
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.hair_style; i++) {
    writePng(join(dir, 'layers', `hair_style_${i}.png`), buildLayerStrip(id, 'hair', i, cell));
  }
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.face; i++) {
    writePng(join(dir, 'layers', `face_${i}.png`), buildLayerStrip(id, 'face', i, cell));
  }

  const rig = makeRig(id, cell);
  writeFileSync(join(dir, 'rig.json'), JSON.stringify(rig, null, 2) + '\n');
  writeFileSync(join(dir, 'atlas.json'), JSON.stringify({
    image: 'sheet.png', cell: rig.cell, anchor: rig.anchor, frames: rig.frames, clips: rig.clips, fallbacks: rig.fallbacks,
  }, null, 2) + '\n');
  writeFileSync(join(dir, 'manifest.json'), JSON.stringify({
    framework: 'pixel-forge', id, kind: 'humanoid', sheet: 'sheet.png', rig: 'rig.json', atlas: 'atlas.json',
    clips: Object.keys(rig.clips), layers: rig.layers, customization: rig.customization,
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
