import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from '../png.mjs';
import { P } from '../palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../../..');

// Matrix spritesheet contract (humanoids):
// - rows = states, columns = frames of that state.
// - frame dimensions/anchors are defined by each entity (per-state).

export const HUMANOID_STATE_ORDER = [
  'idle', 'walk', 'run', 'crouch', 'crawl', 'jump', 'jump_rise', 'jump_fall',
  'fly', 'swim', 'climb', 'dash', 'sixseven_a', 'sixseven_b',
  'attack_1', 'attack_2', 'attack_3', 'kick', 'cast', 'shout', 'hurt', 'dying', 'dead',
];

export const HUMANOID_CLIPS = {
  idle: { frames: ['idle_0', 'idle_1'], frameDuration: 0.4, fps: 2.5, loop: true },
  walk: { frames: ['walk_0', 'walk_1', 'walk_2', 'walk_3'], frameDuration: 0.125, fps: 8, loop: true },
  run: { frames: ['run_0', 'run_1', 'run_2', 'run_3'], frameDuration: 0.1, fps: 10, loop: true },
  crouch: { frames: ['crouch_0'], frameDuration: 1.0, fps: 1, loop: true },
  crawl: { frames: ['crawl_0', 'crawl_1', 'crawl_2', 'crawl_3'], frameDuration: 1 / 7, fps: 7, loop: true },
  jump: { frames: ['jump_rise_0'], frameDuration: 1.0, fps: 1, loop: true },
  jump_rise: { frames: ['jump_rise_0'], frameDuration: 1.0, fps: 1, loop: true },
  jump_fall: { frames: ['jump_fall_0'], frameDuration: 1.0, fps: 1, loop: true },
  fly: { frames: ['fly_0', 'fly_1'], frameDuration: 1 / 6, fps: 6, loop: true },
  swim: { frames: ['swim_0', 'swim_1'], frameDuration: 0.2, fps: 5, loop: true },
  climb: { frames: ['climb_0', 'climb_1'], frameDuration: 0.2, fps: 5, loop: true },
  dash: { frames: ['dash_0'], frameDuration: 1.0, fps: 1, loop: true },
  sixseven_a: { frames: ['sixseven_a_0'], frameDuration: 1.0, fps: 1, loop: true },
  sixseven_b: { frames: ['sixseven_b_0'], frameDuration: 1.0, fps: 1, loop: true },
  attack_1: { frames: ['attack_1_0'], frameDuration: 0.1, fps: 10, loop: false },
  attack_2: { frames: ['attack_2_0'], frameDuration: 0.1, fps: 10, loop: false },
  attack_3: { frames: ['attack_3_0'], frameDuration: 0.1, fps: 10, loop: false },
  kick: { frames: ['kick_0'], frameDuration: 0.1, fps: 10, loop: false },
  cast: { frames: ['cast_0'], frameDuration: 0.125, fps: 8, loop: false },
  shout: { frames: ['shout_0'], frameDuration: 0.125, fps: 8, loop: false },
  hurt: { frames: ['hurt_0'], frameDuration: 1.0, fps: 1, loop: true },
  dying: { frames: ['dying_0'], frameDuration: 0.25, fps: 4, loop: false },
  dead: { frames: ['dead_0'], frameDuration: 1.0, fps: 1, loop: true },
};

export const HUMANOID_FALLBACKS = {
  run: ['walk', 'idle'],
  crouch: ['idle'],
  crawl: ['crouch', 'walk', 'idle'],
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

// Compatibility: flat frame-name list (derived from clip order/state order).
export const HUMANOID_FRAMES = HUMANOID_STATE_ORDER.flatMap((state) => HUMANOID_CLIPS[state]?.frames || []);

export const CUSTOMIZATION_VARIANTS = {
  skin_tone: 8,
  hair_style: 8,
  hair_color: 8,
  face: 8,
  outfit_top: 8,
  outfit_bottom: 8,
  shoes: 8,
};

function writePng(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

function stateSpec(entity, state) {
  if (entity && typeof entity.getStateFrameSpec === 'function') {
    return entity.getStateFrameSpec(state);
  }
  const specs = entity?.STATE_FRAME_SPECS || {};
  return specs[state] || { w: 64, h: 64, anchor: { x: 32, y: 62 } };
}

export function computeMatrixLayout(entity) {
  let y = 0;
  let totalW = 0;
  const rows = [];
  const states = {};
  for (let row = 0; row < HUMANOID_STATE_ORDER.length; row++) {
    const stateName = HUMANOID_STATE_ORDER[row];
    const clip = HUMANOID_CLIPS[stateName];
    const spec = stateSpec(entity, stateName);
    const frames = (clip?.frames || []).map((name, col) => {
      const x = col * spec.w;
      return { name, col, x, y, w: spec.w, h: spec.h, anchor: spec.anchor };
    });
    const rowW = frames.length * spec.w;
    const rowH = spec.h;
    const stateMeta = {
      row,
      y,
      frameW: spec.w,
      frameH: spec.h,
      anchor: spec.anchor,
      frameDuration: clip?.frameDuration ?? (clip?.fps ? 1 / clip.fps : 1),
      fps: clip?.fps ?? 1,
      loop: !!clip?.loop,
      frameCount: frames.length,
      frames,
    };
    rows.push({ state: stateName, ...stateMeta, rowW, rowH });
    states[stateName] = stateMeta;
    y += rowH;
    if (rowW > totalW) totalW = rowW;
  }
  return { statesOrder: [...HUMANOID_STATE_ORDER], rows, states, width: totalW, height: y };
}

function poseFromFrame(frame) {
  const f = frame;
  const base = {
    step: 0, air: 0, crouch: false, crawl: false, run: false, swim: false, fly: false, climb: false, dead: false,
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
  // Low states follow idle/walk logic: same head/face/body structure, lower
  // posture, limbs change most. Their geometry lives in drawLowLayer.
  if (f === 'crouch_0') return { ...base, crouch: true };
  // Crawl = crouch-walk cycle (alternating diagonal gait), same timing as walk.
  if (f === 'crawl_0') return { ...base, crawl: true, crouch: true, armL: -2, armR: 2, legL: 2, legR: -2 };
  if (f === 'crawl_1') return { ...base, crawl: true, crouch: true, armL: -1, armR: 1, legL: 0, legR: 0 };
  if (f === 'crawl_2') return { ...base, crawl: true, crouch: true, armL: 2, armR: -2, legL: -2, legR: 2 };
  if (f === 'crawl_3') return { ...base, crawl: true, crouch: true, armL: 1, armR: -1, legL: 0, legR: 0 };
  if (f === 'jump_rise_0') return { ...base, air: -3, bob: -1, armUpL: true, armUpR: true, legL: -1, legR: 1 };
  if (f === 'jump_fall_0') return { ...base, air: -1, step: 1, bob: 1, armOutL: true, armOutR: true, legL: 1, legR: -1 };
  if (f === 'fly_0') return { ...base, fly: true, air: -2, bob: -1, armOutL: true, armOutR: true, legL: -1, legR: 1 };
  if (f === 'fly_1') return { ...base, fly: true, air: -2, bob: 0, armOutL: true, armOutR: true, legL: 1, legR: -1 };
  if (f === 'swim_0') return { ...base, swim: true, air: -1, bob: 1, lean: 1, armUpL: true, armR: 3, legL: -1, legR: 1 };
  if (f === 'swim_1') return { ...base, swim: true, air: -1, bob: 1, lean: 1, armL: 3, armUpR: true, legL: 1, legR: -1 };
  if (f === 'climb_0') return { ...base, climb: true, bob: 0, armUpL: true, armR: 2, legL: -1, legR: 1 };
  if (f === 'climb_1') return { ...base, climb: true, bob: -1, armL: 2, armUpR: true, legL: 1, legR: -1 };
  if (f === 'dash_0') return { ...base, run: true, step: 1, air: -1, lean: 2, bob: 0, armL: -1, armR: 4, legL: -2, legR: 2 };
  if (f === 'sixseven_a_0') return { ...base, bob: -1, armUpL: true, armR: 3, legL: 0, legR: 0 };
  if (f === 'sixseven_b_0') return { ...base, bob: -1, armL: 3, armUpR: true, legL: 0, legR: 0 };
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

// Low states (crouch / crawl) are authored natively in canonical spaces scaled 2x
// to matrix frames, keeping pixel density consistent with 64x64 standing frames.
// Crouch/kneel is a bit taller than crawling.
const LOW_FRAME_W = 54;
const CRAWL_FRAME_H = 16;  // -> 32px
const CROUCH_FRAME_H = 20; // -> 40px

function lowCanonicalHeight(frame) {
  if (typeof frame === 'string' && frame.startsWith('crouch')) return CROUCH_FRAME_H;
  return CRAWL_FRAME_H;
}

function drawLowLayer(c, frame, charId, layer, variant = 0, ch = lowCanonicalHeight(frame)) {
  const pose = poseFromFrame(frame);
  const isVal = charId === 'valentina';
  const crawl = !!pose.crawl;
  const skin = '#f2c9a0';
  const skinFar = '#d8a878';
  const shirt = isVal ? '#ff6fae' : '#4f7cff';
  const shirtDark = isVal ? '#e0578f' : '#3d63d6';
  const collar = isVal ? '#ff9fc8' : '#7fa0ff';
  const pants = isVal ? '#5a3f8f' : '#2f3a66';
  const pantsFar = isVal ? '#472f72' : '#232b4d';
  const shoe = '#3a2a1a';
  const shoeFar = '#2a1d12';
  const knee = isVal ? '#4a2f6a' : '#263055';
  const kneeFar = isVal ? '#3b2455' : '#1c2340';
  const draw = (cond, fn) => { if (cond) fn(); };
  const stride = (v) => Math.max(-2, Math.min(2, v | 0));

  // Reuse the exact head/face/hair art from the standing poses: render the
  // head-only layer in canonical 32x32 space and blit it at the low head
  // position. Face and hair therefore only change position, never design.
  const HEAD_TMP_X = 8;
  const blitHead = (headLayer, headX, headY) => {
    const tmp = new Canvas(32, 32);
    drawHumanoidLayer(tmp, HEAD_TMP_X - 11, frame, charId, headLayer, variant);
    c.blit(tmp, headX - HEAD_TMP_X, headY - 15);
  };

  const gy = ch - 2; // ground row (shoes/feet bottom)

  if (!crawl) {
    // Crouch: one knee on the ground, the other raised above the hip line.
    const headX = 22;
    const headY = 2;
    const bodyY = headY + 7;
    const hipY = 12;

    draw(layer === 'full' || layer === 'base' || layer === 'skin', () => {
      c.rect(headX, headY, 10, 7, skin);
      c.rect(headX + 2, headY + 7, 5, 1, skin); // neck
      // Arms hanging naturally by the torso.
      c.rect(20, bodyY, 2, 4, skinFar);
      c.rect(19, bodyY + 3, 3, 2, skinFar);
      c.rect(19, bodyY + 4, 2, 2, skinFar); // hand
      c.rect(32, bodyY, 2, 4, skin);
      c.rect(33, bodyY + 3, 3, 2, skin);
      c.rect(34, bodyY + 4, 2, 2, skin); // hand
    });
    draw(layer === 'full' || layer === 'face', () => blitHead('face', headX, headY));
    draw(layer === 'full' || layer === 'outfit_bottom' || layer === 'pants', () => {
      // Hips/waist (hip line reference).
      c.rect(21, hipY, 12, 2, pants);
      // Grounded leg: thigh down to knee touching the ground.
      c.rect(19, hipY + 1, 5, 3, pantsFar); // thigh
      c.rect(18, gy - 3, 5, 3, kneeFar);    // knee on ground (prominent)
      c.rect(14, gy - 1, 6, 1, pantsFar);   // shin/foot tucked back
      c.rect(15, gy - 2, 5, 1, pantsFar);
      // Raised leg: knee held high above the hip line, foot forward on ground.
      c.rect(28, hipY, 5, 2, pants);        // thigh up/forward
      c.rect(32, hipY - 3, 4, 3, knee);      // knee above hip line (prominent)
      c.rect(33, hipY, 3, 5, pants);        // shin down vertically
      c.rect(33, gy - 1, 3, 1, pants);
    });
    draw(layer === 'full' || layer === 'shoes', () => {
      c.rect(12, gy, 6, 2, shoeFar);        // back foot tucked
      c.rect(31, gy, 6, 2, shoe);           // front foot
    });
    draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
      c.rect(20, bodyY - 1, 14, 1, collar);
      c.rect(20, bodyY, 14, 4, shirt);      // upright torso/body
      c.rect(20, bodyY + 3, 14, 1, shirtDark);
      c.rect(19, bodyY, 3, 4, shirt);       // far sleeve
      c.rect(32, bodyY, 3, 4, shirt);       // near sleeve
    });
    draw(layer === 'full' || layer === 'hair', () => blitHead('hair', headX, headY));
    return;
  }

  // Crawl: on hands and knees, compact body. Height oscillates while moving.
  const aN = stride(pose.armR);
  const aF = stride(pose.armL);
  const kN = stride(pose.legR);
  const kF = stride(pose.legL);
  // Bob: body lifts slightly on passing frames, settles on contact frames.
  const bob = (frame === 'crawl_1' || frame === 'crawl_3') ? 0 : -1;
  const headX = 30;
  const headY = 1 + bob;
  const bodyY = 8 + bob;
  const hipY = 8 + bob;
  const limbY = 11 + bob;

  draw(layer === 'full' || layer === 'base' || layer === 'skin', () => {
    c.rect(headX, headY, 10, 7, skin);
    c.rect(headX + 1, headY + 7, 4, 1, skin); // neck to shoulders
    // Far arm: shoulder down to hand planted on ground.
    c.rect(28, bodyY + 1, 2, 2 + (bob === 0 ? 1 : 0), skinFar);
    c.rect(29 + aF, limbY, 2, 2, skinFar);
    c.rect(30 + aF, gy - 1, 4, 2, skinFar);
    // Near arm.
    c.rect(32, bodyY + 1, 2, 2 + (bob === 0 ? 1 : 0), skin);
    c.rect(33 + aN, limbY, 2, 2, skin);
    c.rect(34 + aN, gy - 1, 4, 2, skin);
  });
  draw(layer === 'full' || layer === 'face', () => blitHead('face', headX, headY));
  draw(layer === 'full' || layer === 'outfit_bottom' || layer === 'pants', () => {
    // Hips/seat - compact body mass.
    c.rect(14, hipY, 11, 4, pants);
    c.rect(14, hipY, 11, 1, pantsFar);
    // Far leg: thigh to a clearly bent, prominent knee on ground.
    c.rect(14, limbY, 6, 2, pantsFar);
    c.rect(14 + kF, limbY + 1, 4, 2, kneeFar);
    c.rect(14 + kF, gy - 1, 3, 1, pantsFar);
    // Near leg.
    c.rect(19, limbY, 6, 2, pants);
    c.rect(19 + kN, limbY + 1, 4, 2, knee);
    c.rect(19 + kN, gy - 1, 3, 1, pants);
  });
  draw(layer === 'full' || layer === 'shoes', () => {
    c.rect(12 + kF, gy, 6, 2, shoeFar);
    c.rect(17 + kN, gy, 6, 2, shoe);
  });
  draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
    c.rect(23, bodyY, 11, 1, collar);
    c.rect(23, bodyY + 1, 11, 3, shirt);
    c.rect(23, bodyY + 3, 11, 1, shirtDark);
    c.rect(27, bodyY + 1, 3, 3, shirt); // far shoulder/sleeve
    c.rect(31, bodyY + 1, 3, 3, shirt); // near shoulder/sleeve
  });
  draw(layer === 'full' || layer === 'hair', () => blitHead('hair', headX, headY));
}

function drawHumanoidLayer(c, ox, frame, charId, layer, variant = 0) {
  const pose = poseFromFrame(frame);
  const hurt = frame === 'hurt_0';
  const dead = frame === 'dying_0' || frame === 'dead_0';
  const isVal = charId === 'valentina';
  const low = !!(pose.crouch || pose.crawl);
  const crawl = !!pose.crawl;
  const skinBase = '#f2c9a0';
  const skin = dead ? '#d8d0e8' : hurt ? '#ffd0d0' : skinBase;
  const shirtBase = isVal ? '#ff6fae' : '#4f7cff';
  const pantsBase = isVal ? '#5a3f8f' : '#2f3a66';
  const hairBase = isVal ? '#5a3218' : '#5a3a22';
  const shirt = hurt ? '#ff8a8a' : shirtBase;
  const pants = pantsBase;
  const hair = hairBase;
  const shoe = '#3a2a1a';
  const hairStyle = layer === 'hair' ? variant : 0;
  const faceStyle = layer === 'face' ? variant : 0;
  // Unified humanoid layout (coherent with idle/walk/run): head/face stay the same,
  // only position/height change in low states; limbs/body adapt most.
  // Low states are half-height and drawn in a wider frame; feet anchored near source bottom.
  const y = pose.air + (pose.bob || 0);
  const lean = pose.lean || 0;
  const lowState = !!pose.crouch; // crouch or crawl(rastejando)
  const headY = lowState ? (15 + y) : (6 + y);
  const bodyY = lowState ? (21 + y) : (14 + y);
  const bodyH = lowState ? 5 : 10;
  const armH = lowState ? 4 : 7;
  const legTop = lowState ? 25 : 23;
  const footY = lowState ? 28 : 29;
  const legH = lowState ? 3 : (dead ? 3 : 6);
  const draw = (cond, fn) => { if (cond) fn(); };
  const armRect = (x, _y0, up, out, dy) => {
    if (up) c.rect(ox + x, bodyY - 3 + dy, 2, 8, skin);
    else if (out) {
      if (x > 16) c.rect(ox + x, bodyY + 2 + dy, 7, 3, skin);
      else c.rect(ox + x - 5, bodyY + 2 + dy, 7, 3, skin);
    } else c.rect(ox + x, bodyY + 1 + dy, 2, armH, skin);
  };
  draw(layer === 'full' || layer === 'base' || layer === 'skin', () => {
    // Head (same shape/face features drawn later) only moves down in low states.
    c.rect(ox + 11 + lean, headY, 10, 7, skin);
    c.rect(ox + 13 + lean, headY + 7, 6, 1, skin); // neck
    const ldy = pose.armL || 0;
    const rdy = pose.armR || 0;
    armRect(8 + lean, bodyY, !!pose.armUpL, !!pose.armOutL, ldy);
    armRect(22 + lean, bodyY, !!pose.armUpR, !!pose.armOutR, rdy);
    if (pose.climb) {
      c.rect(ox + 11, 25, 4, pose.legL > 0 ? 3 : 2, skin);
      c.rect(ox + 17, 25, 4, pose.legR > 0 ? 3 : 2, skin);
    } else if (lowState) {
      // Bent legs for crouch/crawl (coherent with upright stance, lower height).
      const lLift = Math.max(-1, Math.min(1, pose.legL || 0));
      const rLift = Math.max(-1, Math.min(1, pose.legR || 0));
      c.rect(ox + 11 + (pose.step < 0 ? -1 : 0), legTop - (lLift < 0 ? 1 : 0), 4, Math.max(2, legH + (lLift > 0 ? 1 : 0)), skin);
      c.rect(ox + 17 + (pose.step > 0 ? 1 : 0), legTop - (rLift < 0 ? 1 : 0), 4, Math.max(2, legH + (rLift > 0 ? 1 : 0)), skin);
    } else {
      const ll = Math.max(1, legH - 1 + (pose.legL || 0));
      const rl = Math.max(1, legH - 1 + (pose.legR || 0));
      c.rect(ox + 11, 24 - (pose.legL < 0 ? 1 : 0), 4, ll, skin);
      c.rect(ox + 17, 24 - (pose.legR < 0 ? 1 : 0), 4, rl, skin);
    }
  });
  draw(layer === 'full' || layer === 'face', () => {
    const hx = ox + lean; const eyeY = headY + 3; const eye = dead ? '#7a6a8a' : P.outline; const mouthY = headY + 6;
    const fs = Math.max(0, Math.min(7, faceStyle | 0));
    if (isVal) {
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
      if (fs === 0) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY, 2, 2, eye); c.rect(hx + 12, eyeY - 1, 3, 1, '#3a2418'); c.rect(hx + 17, eyeY - 1, 3, 1, '#3a2418'); }
      else if (fs === 1) { c.rect(hx + 13, eyeY + 1, 2, 1, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.rect(hx + 12, eyeY, 4, 1, '#4a3422'); c.rect(hx + 16, eyeY, 4, 1, '#4a3422'); }
      else if (fs === 2) { c.rect(hx + 12, eyeY, 3, 1, eye); c.rect(hx + 17, eyeY, 3, 1, eye); c.set(hx + 14, eyeY + 1, eye); c.set(hx + 18, eyeY + 1, eye); }
      else if (fs === 3) { c.rect(hx + 13, eyeY, 2, 2, '#5a3820'); c.rect(hx + 17, eyeY, 2, 2, '#5a3820'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); }
      else if (fs === 4) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY, 2, 2, eye); c.rect(hx + 14, mouthY + 1, 4, 1, '#8a6a4a'); }
      else if (fs === 5) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.rect(hx + 12, eyeY - 1, 2, 1, '#3a2418'); }
      else if (fs === 6) { c.rect(hx + 12, eyeY, 3, 2, eye); c.rect(hx + 17, eyeY, 3, 2, eye); c.rect(hx + 13, mouthY + 1, 6, 1, '#6a4a32'); }
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
    const hx = ox + lean; const hs = Math.max(0, Math.min(7, hairStyle | 0)); const longBack = low ? 3 : 10;
    if (isVal) {
      if (hs === 0) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(10, longBack + 3), hair); c.rect(hx + 21, headY + 2, 2, Math.min(10, longBack + 3), hair); }
      else if (hs === 1) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 15, headY - 2, 2, 2, hair); c.rect(hx + 21, headY + 2, 2, Math.min(8, longBack + 2), hair); c.rect(hx + 22, headY + 8, 1, low ? 0 : 2, hair); }
      else if (hs === 2) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(8, longBack + 2), hair); c.rect(hx + 21, headY + 2, 2, Math.min(8, longBack + 2), hair); }
      else if (hs === 3) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 1, 3, Math.min(6, longBack + 1), hair); c.rect(hx + 20, headY + 1, 3, Math.min(6, longBack + 1), hair); c.rect(hx + 10, headY + 6, 2, low ? 0 : 2, hair); c.rect(hx + 20, headY + 6, 2, low ? 0 : 2, hair); }
      else if (hs === 4) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(5, longBack), hair); c.rect(hx + 21, headY + 2, 2, Math.min(5, longBack), hair); }
      else if (hs === 5) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 9, headY + 2, 2, Math.min(7, longBack + 1), hair); c.rect(hx + 21, headY + 2, 2, Math.min(7, longBack + 1), hair); c.rect(hx + 9, headY + 8, 1, low ? 0 : 2, hair); c.rect(hx + 22, headY + 8, 1, low ? 0 : 2, hair); }
      else if (hs === 6) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 14, headY - 3, 4, 2, hair); c.rect(hx + 9, headY + 2, 2, 4, hair); c.rect(hx + 21, headY + 2, 2, 4, hair); }
      else { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 3, Math.min(9, longBack + 2), hair); c.rect(hx + 20, headY + 2, 3, Math.min(9, longBack + 2), hair); c.rect(hx + 12, headY + 10, 8, low ? 0 : 1, hair); }
    } else {
      if (hs === 0) { /* bald */ }
      else if (hs === 1) { c.rect(hx + 11, headY + 1, 10, 2, hair); c.rect(hx + 10, headY + 2, 1, 1, hair); c.rect(hx + 21, headY + 2, 1, 1, hair); }
      else if (hs === 2) { c.rect(hx + 11, headY + 1, 10, 1, hair); }
      else if (hs === 3) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 12, headY - 1, 2, 1, hair); c.rect(hx + 15, headY - 2, 2, 2, hair); c.rect(hx + 18, headY - 1, 2, 1, hair); }
      else if (hs === 4) { c.rect(hx + 10, headY, 11, 3, hair); c.rect(hx + 9, headY + 1, 2, 2, hair); c.rect(hx + 20, headY, 2, 1, hair); }
      else if (hs === 5) { c.rect(hx + 11, headY, 10, 2, hair); c.rect(hx + 10, headY + 1, 1, 2, hair); c.rect(hx + 21, headY + 1, 1, 2, hair); c.rect(hx + 14, headY - 1, 4, 1, hair); }
      else if (hs === 6) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 1, 2, 3, hair); c.rect(hx + 21, headY + 1, 2, 3, hair); }
      else { c.rect(hx + 10, headY, 12, 3, hair); c.rect(hx + 9, headY + 2, 2, 3, hair); c.rect(hx + 21, headY + 2, 2, 2, hair); }
    }
  });
  draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
    const tx = ox + lean;
    c.rect(tx + 10, bodyY, 12, bodyH, shirt);
    c.rect(tx + 10, bodyY, 12, 1, hurt ? '#ffaaaa' : (isVal ? '#ff9fc8' : '#7fa0ff'));
    // sleeves follow arm pose (same logic as standing, only position changes in low states)
    if (pose.armUpL) c.rect(tx + 8, bodyY - 1, 3, 3, shirt);
    else c.rect(tx + 8, bodyY + 1, 3, 2, shirt);
    if (pose.armUpR) c.rect(tx + 21, bodyY - 1, 3, 3, shirt);
    else if (pose.armOutR) c.rect(tx + 21, bodyY + 2, 3, 2, shirt);
    else c.rect(tx + 21, bodyY + 1, 3, 2, shirt);
  });
  draw(layer === 'full' || layer === 'outfit_bottom' || layer === 'pants', () => {
    if (pose.climb) {
      c.rect(ox + 11, 24, 4, frame === 'climb_1' ? 5 : 4, pants);
      c.rect(ox + 17, 24, 4, frame === 'climb_1' ? 4 : 5, pants);
    } else if (lowState) {
      const lLift = Math.max(-1, Math.min(1, pose.legL || 0));
      const rLift = Math.max(-1, Math.min(1, pose.legR || 0));
      c.rect(ox + 11 + (pose.step < 0 ? -1 : 0), legTop - (lLift < 0 ? 1 : 0), 4, legH + (lLift > 0 ? 1 : 0) + 1, pants);
      c.rect(ox + 17 + (pose.step > 0 ? 1 : 0), legTop - (rLift < 0 ? 1 : 0), 4, legH + (rLift > 0 ? 1 : 0) + 1, pants);
    } else {
      const ly = 23;
      c.rect(ox + 11, ly - (pose.legL < 0 ? 1 : 0), 4, legH + Math.max(0, pose.legL || 0), pants);
      c.rect(ox + 17, ly - (pose.legR < 0 ? 1 : 0), 4, legH + Math.max(0, pose.legR || 0), pants);
    }
  });
  draw(layer === 'full' || layer === 'shoes', () => {
    const fy = footY;
    if (dead) { c.rect(ox + 8, fy - 1, 8, 2, shoe); c.rect(ox + 16, fy - 1, 8, 2, shoe); return; }
    if (lowState) {
      const ls = pose.legL < 0 ? -1 : 0;
      const rs = pose.legR < 0 ? -1 : 0;
      c.rect(ox + 10 + (pose.step < 0 ? -1 : 0), fy + ls, 5, 2, shoe);
      c.rect(ox + 17 + (pose.step > 0 ? 1 : 0), fy + rs, 5, 2, shoe);
      return;
    }
    const ls = pose.legL < 0 ? -1 : 0; const rs = pose.legR < 0 ? -1 : 0;
    c.rect(ox + 10, fy + ls, 5, 2, shoe);
    c.rect(ox + 17 + (frame === 'kick_0' ? 3 : 0), fy + rs + (frame === 'kick_0' ? -3 : 0), 6, 2, shoe);
    if (frame === 'kick_0') c.rect(ox + 22, fy - 4, 2, 3, pants);
  });
  if (layer === 'full' || layer === 'action_overlay') {
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

function scaleCanvas(src, w, h) {
  if (src.w === w && src.h === h) return src;
  const out = new Canvas(w, h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const sx = Math.min(src.w - 1, Math.floor((x * src.w) / w));
      const sy = Math.min(src.h - 1, Math.floor((y * src.h) / h));
      const i = (sy * src.w + sx) * 4;
      out.set(x, y, [src.data[i], src.data[i + 1], src.data[i + 2], src.data[i + 3]]);
    }
  }
  return out;
}

function cropCanvas(src, sx, sy, w, h) {
  const out = new Canvas(w, h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const px = sx + x;
      const py = sy + y;
      if (px < 0 || py < 0 || px >= src.w || py >= src.h) continue;
      const i = (py * src.w + px) * 4;
      out.set(x, y, [src.data[i], src.data[i + 1], src.data[i + 2], src.data[i + 3]]);
    }
  }
  return out;
}

function buildFrameCanvas(entity, frame, layer, variant, fw, fh) {
  const id = entity.ENTITY_ID || entity.id;

  // Low states (crouch/crawl): drawn natively in canonical low space and scaled.
  if (typeof frame === 'string' && (frame.startsWith('crouch') || frame.startsWith('crawl'))) {
    const ch = lowCanonicalHeight(frame);
    const low = new Canvas(LOW_FRAME_W, ch);
    drawLowLayer(low, frame, id, layer, variant, ch);
    if (fw === LOW_FRAME_W * 2 && fh === ch * 2) return low.scale2x();
    return scaleCanvas(low, fw, fh);
  }

  // Draw in canonical 32x32 humanoid space, then adapt to state-specific frame size.
  const src = new Canvas(32, 32);
  drawHumanoidLayer(src, 0, frame, id, layer, variant);

  // Standing/default humanoid frame.
  if (fw === 64 && fh === 64) return src.scale2x();

  if (fh < 64) {
    if (fh <= 32) {
      const cropped = cropCanvas(src, 0, 13, 32, 16);
      const scaled = scaleCanvas(cropped, 64, 32);
      const out = new Canvas(fw, fh);
      out.blit(scaled, Math.floor((fw - scaled.w) / 2), fh - scaled.h);
      return out;
    }
    const cropped = cropCanvas(src, 0, 8, 32, 24);
    return scaleCanvas(cropped, fw, fh);
  }

  // Generic fallback: scale full source to requested frame size.
  return scaleCanvas(src, fw, fh);
}

function buildMatrixSheet(entity, layer, variant = 0, layout = computeMatrixLayout(entity)) {
  const c = new Canvas(layout.width, layout.height);
  for (const row of layout.rows) {
    for (const frame of row.frames) {
      const fc = buildFrameCanvas(entity, frame.name, layer, variant, row.frameW, row.frameH);
      for (let y = 0; y < row.frameH; y++) for (let x = 0; x < row.frameW; x++) {
        const si = (y * fc.w + x) * 4;
        if (si + 3 >= fc.data.length) continue;
        c.set(frame.x + x, frame.y + y, [fc.data[si], fc.data[si + 1], fc.data[si + 2], fc.data[si + 3]]);
      }
    }
  }
  return c;
}

function makeRig(entity, layout) {
  return {
    framework: 'pixel-forge',
    version: 3,
    layout: 'state_matrix',
    id: entity.ENTITY_ID || entity.id,
    kind: 'humanoid',
    image: 'sheet.png',
    atlas: { width: layout.width, height: layout.height, statesOrder: layout.statesOrder },
    states: layout.states,
    clips: HUMANOID_CLIPS,
    fallbacks: HUMANOID_FALLBACKS,
    layers: ['base', 'face', 'hair', 'outfit_bottom', 'shoes', 'outfit_top', 'headgear', 'accessory', 'weapon'],
    customization: {
      variants: CUSTOMIZATION_VARIANTS,
      layer_variants: {
        hair: Array.from({ length: CUSTOMIZATION_VARIANTS.hair_style }, (_, i) => `layers/hair_style_${i}.png`),
        face: Array.from({ length: CUSTOMIZATION_VARIANTS.face }, (_, i) => `layers/face_${i}.png`),
      },
    },
  };
}

export function buildHumanoidEntity(entity) {
  const id = entity.ENTITY_ID || entity.id;
  const dir = join(ROOT, 'assets/entities', id);
  mkdirSync(join(dir, 'layers'), { recursive: true });
  // Per-entity frame dimensions logic defines the matrix layout.
  const layout = computeMatrixLayout(entity);

  const defaultHair = entity.DEFAULT_HAIR_STYLE ?? 0;
  const defaultFace = entity.DEFAULT_FACE ?? 0;

  writePng(join(dir, 'sheet.png'), buildMatrixSheet(entity, 'full', 0, layout));
  for (const layer of ['base', 'outfit_top', 'outfit_bottom', 'shoes']) {
    writePng(join(dir, 'layers', `${layer}.png`), buildMatrixSheet(entity, layer, 0, layout));
  }
  writePng(join(dir, 'layers', 'hair.png'), buildMatrixSheet(entity, 'hair', defaultHair, layout));
  writePng(join(dir, 'layers', 'face.png'), buildMatrixSheet(entity, 'face', defaultFace, layout));
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.hair_style; i++) {
    writePng(join(dir, 'layers', `hair_style_${i}.png`), buildMatrixSheet(entity, 'hair', i, layout));
  }
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.face; i++) {
    writePng(join(dir, 'layers', `face_${i}.png`), buildMatrixSheet(entity, 'face', i, layout));
  }

  const rig = makeRig(entity, layout);
  writeFileSync(join(dir, 'rig.json'), JSON.stringify(rig, null, 2) + '\n');
  writeFileSync(join(dir, 'atlas.json'), JSON.stringify({
    framework: 'pixel-forge',
    layout: 'state_matrix',
    image: 'sheet.png',
    width: layout.width,
    height: layout.height,
    statesOrder: layout.statesOrder,
    states: layout.states,
    clips: HUMANOID_CLIPS,
    fallbacks: HUMANOID_FALLBACKS,
  }, null, 2) + '\n');
  writeFileSync(join(dir, 'manifest.json'), JSON.stringify({
    framework: 'pixel-forge',
    version: 3,
    id,
    kind: 'humanoid',
    layout: 'state_matrix',
    sheet: 'sheet.png',
    rig: 'rig.json',
    atlas: 'atlas.json',
    statesOrder: layout.statesOrder,
    clips: Object.keys(rig.clips),
    layers: rig.layers,
    customization: rig.customization,
  }, null, 2) + '\n');
  return [`assets/entities/${id}/sheet.png`, `assets/entities/${id}/rig.json`, `assets/entities/${id}/atlas.json`];
}

function readPngSize(path) {
  const b = readFileSync(path);
  if (b.length < 24 || b[0] !== 137 || b[1] !== 80 || b[2] !== 78 || b[3] !== 71) throw new Error('invalid PNG');
  return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}

export function validateHumanoidEntity(entity, errors = []) {
  const id = entity.ENTITY_ID || entity.id;
  const expected = computeMatrixLayout(entity);
  for (const name of ['sheet.png', 'atlas.json', 'manifest.json', 'rig.json']) {
    const p = join(ROOT, 'assets/entities', id, name);
    if (!existsSync(p)) errors.push(`entity ${id}: missing ${name}`);
  }
  const rigPath = join(ROOT, 'assets/entities', id, 'rig.json');
  if (!existsSync(rigPath)) return errors;
  let rig = null;
  try { rig = JSON.parse(readFileSync(rigPath, 'utf8')); } catch (e) { errors.push(`entity ${id}: invalid rig.json (${e.message})`); return errors; }

  for (const clip of ['idle', 'walk', 'jump_rise', 'attack_1', 'sixseven_a', 'hurt', 'dead']) {
    if (!rig.clips?.[clip]) errors.push(`entity ${id}: rig missing clip ${clip}`);
  }
  if (!Array.isArray(rig.layers)) errors.push(`entity ${id}: rig missing layers`);
  if (rig.layout !== 'state_matrix') { errors.push(`entity ${id}: expected layout state_matrix`); return errors; }

  const order = rig.atlas?.statesOrder || rig.statesOrder || [];
  if (order.join(',') !== expected.statesOrder.join(',')) errors.push(`entity ${id}: invalid statesOrder`);
  if (!rig.states || typeof rig.states !== 'object') { errors.push(`entity ${id}: matrix rig missing states metadata`); return errors; }

  for (const state of expected.statesOrder) {
    const s = rig.states[state];
    const es = expected.states[state];
    if (!s) { errors.push(`entity ${id}: missing state row '${state}'`); continue; }
    if (s.row !== es.row) errors.push(`entity ${id}: state '${state}' has wrong row index`);
    if (s.frameCount !== es.frameCount) errors.push(`entity ${id}: state '${state}' has wrong frame count`);
    if (!(s.frameDuration > 0 || s.fps > 0)) errors.push(`entity ${id}: state '${state}' missing frame duration/fps`);
    if (s.frameW !== es.frameW || s.frameH !== es.frameH) errors.push(`entity ${id}: state '${state}' frame dimensions must match entity contract`);
    if (!Array.isArray(s.frames) || s.frames.length !== es.frameCount) { errors.push(`entity ${id}: state '${state}' missing frame rects`); continue; }
    for (let i = 0; i < s.frames.length; i++) {
      const f = s.frames[i]; const ef = es.frames[i];
      if (f.x !== ef.x || f.y !== ef.y || f.w !== ef.w || f.h !== ef.h) { errors.push(`entity ${id}: state '${state}' frame ${i} rect mismatch`); break; }
    }
  }

  for (const rel of ['sheet.png', 'layers/base.png', 'layers/face.png', 'layers/hair.png', 'layers/outfit_top.png', 'layers/outfit_bottom.png', 'layers/shoes.png']) {
    const p = join(ROOT, 'assets/entities', id, rel);
    if (!existsSync(p)) { errors.push(`entity ${id}: missing ${rel}`); continue; }
    try {
      const sz = readPngSize(p);
      if (sz.w !== expected.width || sz.h !== expected.height) errors.push(`entity ${id}: ${rel} size ${sz.w}x${sz.h} does not match matrix ${expected.width}x${expected.height}`);
    } catch (e) { errors.push(`entity ${id}: invalid PNG ${rel} (${e.message})`); }
  }
  return errors;
}
