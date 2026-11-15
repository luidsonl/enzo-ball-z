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
  'attack_1', 'attack_2', 'attack_3', 'kick', 'cast', 'shout', 'dying', 'dead',
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
  dying: ['dead', 'idle'],
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
  if (f === 'dying_0') return { ...base, crouch: true, dead: true, bob: 3, lean: -1, armL: 3, armR: 3, legL: 0, legR: 0 };
  if (f === 'dead_0') return { ...base, crouch: true, dead: true, bob: 4, lean: -2, armL: 4, armR: 4, legL: 0, legR: 0 };
  return base;
}

// Low states (crouch / crawl) are authored natively in canonical spaces scaled 2x
// to matrix frames, keeping pixel density consistent with 64x64 standing frames.
// Crouch/kneel is a bit taller than crawling.
const LOW_FRAME_W = 54;
// Content rows used by the low poses (feet on row 12, head top on row 0).
const LOW_CONTENT_H = 14;
// Low frames add two rows of headroom above the content so tall hair styles
// (which draw up to two rows above the head) are never clipped flat at the top
// edge of the frame. 16 canonical rows = 32px, exactly half a standing frame.
const CRAWL_FRAME_H = 16;
const CROUCH_FRAME_H = 16;

function lowCanonicalHeight(frame) {
  if (typeof frame === 'string' && frame.startsWith('crouch')) return CROUCH_FRAME_H;
  return CRAWL_FRAME_H;
}

// Draws into `canvas` shifted down by `dy` rows. Low poses are authored in a
// compact content space and blitted into a taller frame so hair drawn above the
// head still fits inside the exported cell.
function offsetView(canvas, dy) {
  if (!dy) return canvas;
  return {
    w: canvas.w,
    h: canvas.h,
    set: (x, y, color) => canvas.set(x, y + dy, color),
    rect: (x, y, w, h, color) => canvas.rect(x, y + dy, w, h, color),
    clear: (x, y, w, h) => canvas.clear(x, y + dy, w, h),
  };
}

// ---------------------------------------------------------------------------
// Shared pixel-art helpers
// ---------------------------------------------------------------------------

// Draws a thick pixel line. Limbs are intentionally blocky (1xN / Nx1 chunks)
// so the 2x upscale keeps a crisp pixel-art silhouette.
function limb(c, x0, y0, x1, y1, thickness, color) {
  const dx = x1 - x0;
  const dy = y1 - y0;
  const steps = Math.max(Math.abs(dx), Math.abs(dy));
  const horizontalRun = Math.abs(dx) > Math.abs(dy);
  const offset = Math.floor((thickness - 1) / 2);
  for (let i = 0; i <= steps; i += 1) {
    const t = steps === 0 ? 0 : i / steps;
    const x = Math.round(x0 + dx * t);
    const y = Math.round(y0 + dy * t);
    if (horizontalRun) c.rect(x, y - offset, 1, thickness, color);
    else c.rect(x - offset, y, thickness, 1, color);
  }
}

// Small open hand with a wrist/finger shadow.
function handShape(c, x, y, color, shade) {
  c.rect(x, y, 3, 2, color);
  c.set(x, y + 1, shade);
  c.set(x + 2, y + 1, shade);
}

// Two-tone shoe: upper + dark sole. `toe` extends the toe cap forward.
function shoeShape(c, x, y, w, color, sole, toe = 1) {
  c.rect(x, y, w, 1, color);
  c.rect(x, y + 1, w, 1, sole);
  if (toe > 0) c.set(x + w, y, color);
  if (toe > 0) c.set(x + w, y + 1, sole);
}

// Tapered torso/spine block (wider shoulders, narrower waist).
function tapered(c, xTop, yTop, wTop, xBottom, yBottom, wBottom, color) {
  const h = yBottom - yTop;
  for (let i = 0; i < h; i += 1) {
    const t = h <= 1 ? 0 : i / (h - 1);
    const x = Math.round(xTop + (xBottom - xTop) * t);
    const w = Math.max(1, Math.round(wTop + (wBottom - wTop) * t));
    c.rect(x, yTop + i, w, 1, color);
  }
}

// Blocky two-segment arm: upper arm, elbow bridge, forearm. Keeps pixel-art
// steps crisp instead of producing one-pixel staircases.
function blockArm(c, arm, thickness, color) {
  const upperH = Math.max(1, arm.elbowY - arm.shoulderY + 1);
  c.rect(arm.shoulderX - 1, arm.shoulderY, thickness, upperH, color);
  const bridgeX = Math.min(arm.shoulderX, arm.handX) - 1;
  const bridgeW = thickness + Math.abs(arm.handX - arm.shoulderX);
  c.rect(bridgeX, arm.elbowY, bridgeW, 2, color);
  c.rect(arm.handX - 1, arm.elbowY + 1, thickness, Math.max(1, arm.handY - arm.elbowY), color);
}

// Blocky two-segment leg. Expects kneeY >= hipY and ankleY >= kneeY.
function blockLeg(c, leg, thickness, color) {
  const thighH = Math.max(1, leg.kneeY - leg.hipY + 1);
  c.rect(leg.hipX - 1, leg.hipY, thickness, thighH, color);
  const bridgeX = Math.min(leg.hipX, leg.kneeX, leg.ankleX) - 1;
  const spread = Math.max(Math.abs(leg.kneeX - leg.hipX), Math.abs(leg.ankleX - leg.kneeX));
  c.rect(bridgeX, leg.kneeY, thickness + spread, 2, color);
  c.rect(leg.ankleX - 1, leg.kneeY + 1, thickness, Math.max(1, leg.ankleY - leg.kneeY), color);
}

function drawFaceAt(c, hx, headY, frame, charId, variant = 0, expression = 'normal') {
  const isVal = charId === 'valentina';
  const eyeY = headY + 3;
  const mouthY = headY + 6;
  const fs = Math.max(0, Math.min(7, variant | 0));
  const expr = expression || 'normal';
  const eye = (expr === 'cry') ? '#2d4f8f' : '#1f1a17';

  // Base face features by style.
  if (isVal) {
    if (fs === 0) { c.rect(hx + 13, eyeY, 2, 2, '#3a2418'); c.rect(hx + 17, eyeY, 2, 2, '#3a2418'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); c.rect(hx + 12, eyeY + 2, 1, 1, '#e8a0b0'); c.rect(hx + 19, eyeY + 2, 1, 1, '#e8a0b0'); }
    else if (fs === 1) { c.rect(hx + 13, eyeY, 2, 2, '#5a3820'); c.rect(hx + 17, eyeY, 2, 2, '#5a3820'); c.rect(hx + 12, eyeY - 1, 3, 1, '#3a2418'); c.rect(hx + 17, eyeY - 1, 3, 1, '#3a2418'); }
    else if (fs === 2) { c.rect(hx + 13, eyeY, 2, 2, '#ffffff'); c.rect(hx + 17, eyeY, 2, 2, '#ffffff'); c.set(hx + 14, eyeY + 1, '#5a3820'); c.set(hx + 18, eyeY + 1, '#5a3820'); c.rect(hx + 12, eyeY + 2, 2, 1, '#f0b8c0'); c.rect(hx + 18, eyeY + 2, 2, 1, '#f0b8c0'); }
    else if (fs === 3) { c.rect(hx + 13, eyeY, 2, 1, eye); c.rect(hx + 17, eyeY, 2, 1, eye); c.set(hx + 13, eyeY + 1, eye); c.set(hx + 18, eyeY + 1, eye); }
    else if (fs === 4) { c.rect(hx + 13, eyeY, 2, 2, '#6b4226'); c.rect(hx + 17, eyeY, 2, 2, '#6b4226'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 18, eyeY, '#ffffff'); }
    else if (fs === 5) { c.rect(hx + 13, eyeY, 2, 2, eye); c.rect(hx + 17, eyeY + 1, 2, 1, eye); c.set(hx + 19, eyeY + 2, '#f0b8c0'); }
    else if (fs === 6) { c.rect(hx + 12, eyeY, 3, 2, '#4a2818'); c.rect(hx + 17, eyeY, 3, 2, '#4a2818'); c.set(hx + 14, eyeY, '#ffffff'); c.set(hx + 19, eyeY, '#ffffff'); }
    else { c.rect(hx + 13, eyeY, 2, 2, '#7a4a28'); c.rect(hx + 17, eyeY, 2, 2, '#7a4a28'); c.rect(hx + 12, eyeY - 1, 1, 1, '#3a2418'); c.rect(hx + 19, eyeY - 1, 1, 1, '#3a2418'); }
    // Default mouth.
    if (fs === 0 || fs === 2 || fs === 4) { c.rect(hx + 14, mouthY, 4, 1, '#e07a8a'); c.set(hx + 14, mouthY - 1, '#e07a8a'); c.set(hx + 17, mouthY - 1, '#e07a8a'); }
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
    // Default mouth.
    if (fs === 2) c.rect(hx + 14, mouthY, 4, 1, '#1f1a17');
    else c.rect(hx + 15, mouthY - 1, 2, 1, '#b07060');
  }

  // Shout overlay keeps its open mouth.
  if (frame === 'shout_0' && expr === 'normal') {
    c.clear(hx + 14, mouthY - 1, 4, 3);
    c.rect(hx + 14, mouthY - 1, 4, 3, '#4a2030');
  }

  // Expression overlays (3 extra face types): blink, pain, cry.
  if (expr === 'blink') {
    c.clear(hx + 12, eyeY - 1, 8, 4);
    c.rect(hx + 13, eyeY + 1, 2, 1, '#3a2418');
    c.rect(hx + 17, eyeY + 1, 2, 1, '#3a2418');
  } else if (expr === 'pain') {
    c.clear(hx + 12, eyeY - 1, 8, 4);
    c.rect(hx + 13, eyeY, 1, 1, eye); c.rect(hx + 14, eyeY + 1, 1, 1, eye);
    c.rect(hx + 18, eyeY, 1, 1, eye); c.rect(hx + 17, eyeY + 1, 1, 1, eye);
    c.rect(hx + 12, eyeY - 1, 3, 1, '#8a4a4a');
    c.rect(hx + 17, eyeY - 1, 3, 1, '#8a4a4a');
    c.clear(hx + 13, mouthY - 1, 6, 4);
    c.rect(hx + 14, mouthY - 1, 4, 2, '#4a2030');
    c.rect(hx + 15, mouthY, 2, 1, '#d07070');
  } else if (expr === 'cry') {
    c.clear(hx + 12, eyeY - 1, 8, 5);
    c.rect(hx + 13, eyeY, 1, 1, eye); c.rect(hx + 14, eyeY + 1, 1, 1, eye);
    c.rect(hx + 18, eyeY, 1, 1, eye); c.rect(hx + 17, eyeY + 1, 1, 1, eye);
    c.rect(hx + 12, eyeY - 1, 3, 1, '#3a5f9a');
    c.rect(hx + 17, eyeY - 1, 3, 1, '#3a5f9a');
    c.rect(hx + 12, eyeY + 2, 1, 2, '#6fb7ff'); // tear
    c.rect(hx + 19, eyeY + 2, 1, 2, '#6fb7ff'); // tear
    c.clear(hx + 13, mouthY - 1, 6, 4);
    c.rect(hx + 14, mouthY, 4, 2, '#5a3a3a');
    c.rect(hx + 15, mouthY, 2, 1, '#8a5a5a');
  }
}

function drawHairAt(c, hx, headY, frame, charId, variant = 0, low = false) {
  const isVal = charId === 'valentina';
  // Neutral, desaturated base so runtime hair_color tint can colorize accurately.
  const hair = '#d6d6d6';
  const hs = Math.max(0, Math.min(7, variant | 0));
  const longBack = low ? 3 : 10;
  if (isVal) {
    if (hs === 0) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(10, longBack + 3), hair); c.rect(hx + 21, headY + 2, 2, Math.min(10, longBack + 3), hair); }
    else if (hs === 1) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 15, headY - 2, 2, 2, hair); c.rect(hx + 21, headY + 2, 2, Math.min(8, longBack + 2), hair); c.rect(hx + 22, headY + 8, 1, low ? 0 : 2, hair); }
    else if (hs === 2) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(8, longBack + 2), hair); c.rect(hx + 21, headY + 2, 2, Math.min(8, longBack + 2), hair); }
    else if (hs === 3) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 1, 3, Math.min(6, longBack + 1), hair); c.rect(hx + 20, headY + 1, 3, Math.min(6, longBack + 1), hair); c.rect(hx + 10, headY + 6, 2, low ? 0 : 2, hair); c.rect(hx + 20, headY + 6, 2, low ? 0 : 2, hair); }
    else if (hs === 4) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 9, headY + 2, 2, Math.min(5, longBack), hair); c.rect(hx + 21, headY + 2, 2, Math.min(5, longBack), hair); }
    else if (hs === 5) { c.rect(hx + 11, headY, 10, 3, hair); c.rect(hx + 9, headY + 2, 2, Math.min(7, longBack + 1), hair); c.rect(hx + 21, headY + 2, 2, Math.min(7, longBack + 1), hair); c.rect(hx + 9, headY + 8, 1, low ? 0 : 2, hair); c.rect(hx + 22, headY + 8, 1, low ? 0 : 2, hair); }
    else if (hs === 6) { c.rect(hx + 10, headY - 1, 12, 4, hair); c.rect(hx + 14, headY - 3, 4, 2, hair); c.rect(hx + 9, headY + 2, 2, 4, hair); c.rect(hx + 21, headY + 2, 2, 4, hair); }
    else {
      // Long wavy hair with open lower ends (no bottom-connecting line).
      c.rect(hx + 10, headY - 1, 12, 4, hair);
      c.rect(hx + 9, headY + 2, 3, Math.min(8, longBack + 2), hair);
      c.rect(hx + 20, headY + 2, 3, Math.min(8, longBack + 2), hair);
      if (!low) {
        c.rect(hx + 8, headY + 5, 1, 2, hair);
        c.rect(hx + 23, headY + 5, 1, 2, hair);
        c.rect(hx + 9, headY + 8, 2, 1, hair);
        c.rect(hx + 21, headY + 8, 2, 1, hair);
        c.rect(hx + 10, headY + 9, 1, 2, hair);
        c.rect(hx + 21, headY + 9, 1, 2, hair);
      }
    }
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
}

// ---------------------------------------------------------------------------
// Walking / running gaits
// ---------------------------------------------------------------------------
// Four-frame cycle per clip: contact, passing, opposite contact, opposite
// passing. Values are in canonical pixels (2x at export time).
// Contact frames carry the full stride; passing frames bring the feet under
// the body and put the arms at the neutral midpoint. `step` is the foot offset
// from the hip (3/4 stride), `lift` raises the foot, and arms swing opposite
// the same-side leg (`down` when that leg is forward, `up` when it is back).
// Front-facing gait: feet stay under the torso and alternate lifting instead
// of splaying sideways. `step` is only a small weight shift; `lift` carries
// the visible motion.
const WALK_GAIT = {
  walk_0: { bob: 0,  lean: 0, near: { step: 1,  lift: 0 }, far: { step: -1, lift: 2 }, nearArm: 'down', farArm: 'up' },
  walk_1: { bob: -1, lean: 0, near: { step: 0,  lift: 0 }, far: { step: 0,  lift: 3 }, nearArm: 'mid',  farArm: 'mid' },
  walk_2: { bob: 0,  lean: 0, near: { step: -1, lift: 2 }, far: { step: 1,  lift: 0 }, nearArm: 'up',   farArm: 'down' },
  walk_3: { bob: -1, lean: 0, near: { step: 0,  lift: 3 }, far: { step: 0,  lift: 0 }, nearArm: 'mid',  farArm: 'mid' },
};

const RUN_GAIT = {
  run_0: { bob: -1, lean: 1, near: { step: 2,  lift: 0 }, far: { step: -2, lift: 2 }, nearArm: 'down', farArm: 'up' },
  run_1: { bob: -2, lean: 2, near: { step: 0,  lift: 2 }, far: { step: 0,  lift: 4 }, nearArm: 'mid',  farArm: 'mid' },
  run_2: { bob: -1, lean: 1, near: { step: -2, lift: 2 }, far: { step: 2,  lift: 0 }, nearArm: 'up',   farArm: 'down' },
  run_3: { bob: -2, lean: 2, near: { step: 0,  lift: 4 }, far: { step: 0,  lift: 2 }, nearArm: 'mid',  farArm: 'mid' },
};

// Computes the shoe contact box for a stepping leg. Feet stay on the ground
// line (y=29); `bob` only stretches the leg because the torso moves.
function legContact(hipLeft, spec, bob) {
  return {
    footLeft: hipLeft + spec.step,
    footTop: 29 - spec.lift,
    thighY: 24 + bob,
  };
}

// Thin, swinging arm. `mode` bends the forearm toward the torso so the swing
// reads in a 3/4 view without the bulky horizontal slabs the old build had.
function drawArmPose(c, shoulderX, shoulderY, side, mode, color, shade) {
  // `side` is +1 for the near (right) arm and -1 for the far (left) arm, so the
  // raised forearm bends outward and stays visible outside the torso.
  let handY = shoulderY + 6;
  let handX = shoulderX;
  if (mode === 'up') { handY = shoulderY + 2; handX = shoulderX + side * 2; }
  else if (mode === 'mid') { handY = shoulderY + 5; handX = shoulderX + side * 1; }
  c.rect(shoulderX, shoulderY, 2, 4, color);
  if (handY > shoulderY + 4) {
    const lo = Math.min(shoulderX, handX);
    const hi = Math.max(shoulderX + 2, handX + 2);
    c.rect(lo, shoulderY + 3, hi - lo, 2, color);
    c.rect(handX, shoulderY + 4, 2, handY - (shoulderY + 4), color);
  } else {
    const lo = Math.min(shoulderX, handX);
    const hi = Math.max(shoulderX + 2, handX + 2);
    c.rect(lo, handY, hi - lo, 2, color);
  }
  handShape(c, handX, handY - 1, color, shade);
}

// Blocky stepped leg: vertical thigh, one-row knee bridge, offset shin.
function drawBlockLeg(c, hipLeft, spec, bob, pantsColor, kneeColor) {
  const { footLeft, footTop, thighY } = legContact(hipLeft, spec, bob);
  const shinY = thighY + 3;
  c.rect(hipLeft, thighY, 4, 3, pantsColor);
  const lo = Math.min(hipLeft, footLeft);
  const hi = Math.max(hipLeft + 4, footLeft + 4);
  c.rect(lo, shinY - 1, hi - lo, 1, pantsColor);
  const shinH = Math.max(0, footTop - shinY);
  if (shinH > 0) c.rect(footLeft, shinY, 4, shinH, pantsColor);
  c.rect(footLeft - 1, shinY - 1, 2, 1, kneeColor);
  return { footLeft, footTop };
}

function drawGaitLayer(c, frame, charId, layer, variant = 0, expression = 'normal') {
  const gait = WALK_GAIT[frame] || RUN_GAIT[frame];
  if (!gait) return;
  const isVal = charId === 'valentina';
  const skin = '#f2c9a0';
  const skinFar = '#d8a878';
  const handShade = '#b8895c';
  // Neutral, desaturated bases so runtime outfit tints can colorize accurately.
  const shirt = '#d9d9d9';
  const shirtDark = '#a6a6a6';
  const shirtLight = '#f2f2f2';
  const pants = '#bfbfbf';
  const pantsFar = '#999999';
  const shoe = '#3a2a1a';
  const shoeFar = '#2a1d12';
  const sole = '#1d140d';
  const knee = '#8c8c8c';
  const kneeFar = '#737373';

  const bob = gait.bob;
  const lean = gait.lean;
  const headX = 11 + lean;
  const headY = 6 + bob;
  const torsoX = 10 + lean;
  const torsoY = 14 + bob;
  const torsoW = 12;
  const torsoH = 10;
  const shoulderY = torsoY + 1;
  const hipY = torsoY + torsoH;
  const farShoulderX = torsoX - 2;
  const nearShoulderX = torsoX + torsoW;
  // Far leg is the left/back leg; near leg is the right/front leg. Shifting the
  // hips with the lean keeps the legs underneath the torso in run frames.
  const legFarHipX = 11 + lean;
  const legNearHipX = 17 + lean;
  const legFar = legContact(legFarHipX, gait.far, bob);
  const legNear = legContact(legNearHipX, gait.near, bob);

  if (layer === 'full' || layer === 'base' || layer === 'skin') {
    // Far arm and leg sit behind the body.
    drawArmPose(c, farShoulderX, shoulderY, -1, gait.farArm, skinFar, handShade);
    drawBlockLeg(c, legFarHipX, gait.far, bob, skinFar, '#b8895c');
    drawBlockLeg(c, legNearHipX, gait.near, bob, skin, skinFar);
    // Head, neck, and a soft chin shadow.
    c.rect(headX, headY, 10, 7, skin);
    c.rect(headX + 2, headY + 7, 6, 1, skin);
    c.rect(headX + 3, headY + 7, 4, 1, skinFar);
    // Near arm in front of the torso.
    drawArmPose(c, nearShoulderX, shoulderY, 1, gait.nearArm, skin, skinFar);
  }

  if (layer === 'full' || layer === 'face') drawFaceAt(c, headX - 11, headY, frame, charId, variant, expression);

  if (layer === 'full' || layer === 'outfit_bottom' || layer === 'pants') {
    // Waistband bridging the torso and both legs.
    c.rect(torsoX, hipY - 2, torsoW, 2, pants);
    c.rect(torsoX, hipY - 2, torsoW, 1, pantsFar);
    drawBlockLeg(c, legFarHipX, gait.far, bob, pantsFar, kneeFar);
    drawBlockLeg(c, legNearHipX, gait.near, bob, pants, knee);
  }

  if (layer === 'full' || layer === 'shoes') {
    shoeShape(c, legFar.footLeft - 1, legFar.footTop, 6, shoeFar, sole, 0);
    shoeShape(c, legNear.footLeft - 1, legNear.footTop, 7, shoe, sole, 1);
    const planted = gait.near.lift === 0 ? legNear : legFar;
    c.rect(planted.footLeft - 2, 31, 8, 1, '#00000022');
  }

  if (layer === 'full' || layer === 'outfit_top' || layer === 'shirt') {
    // Far sleeve behind the torso, near sleeve in front.
    c.rect(farShoulderX, shoulderY, 2, 3, shirtDark);
    c.rect(torsoX, torsoY, torsoW, torsoH, shirt);
    c.rect(torsoX, torsoY, torsoW, 1, shirtLight);
    c.rect(torsoX, torsoY + 1, 1, torsoH - 2, shirtDark);
    c.rect(torsoX + torsoW - 1, torsoY + 1, 1, torsoH - 2, shirtLight);
    c.rect(torsoX, torsoY + torsoH - 1, torsoW, 1, shirtDark);
    c.rect(torsoX + 5, torsoY, 2, 1, shirtDark);
    c.rect(nearShoulderX, shoulderY, 2, 3, shirt);
    c.rect(nearShoulderX, shoulderY + 2, 2, 1, shirtDark);
  }
  // Body clothing details (separately tintable for customization/equipment).
  if (layer === 'full' || layer === 'outfit_top_detail1') {
    c.rect(torsoX + 1, torsoY + 4, torsoW - 2, 2, '#d9d9d9'); // chest stripe
  }
  if (layer === 'full' || layer === 'outfit_top_detail2') {
    c.rect(torsoX + 4, torsoY, 4, 1, '#e6e6e6'); // collar detail
    c.rect(farShoulderX, shoulderY + 2, 2, 1, '#d9d9d9');
    c.rect(nearShoulderX, shoulderY + 2, 2, 1, '#d9d9d9');
  }

  if (layer === 'full' || layer === 'hair') drawHairAt(c, headX - 11, headY, frame, charId, variant, false);
}

// ---------------------------------------------------------------------------
// Dash: a single explosive lunge pose (contract keeps one frame).
// ---------------------------------------------------------------------------
function drawDashLayer(c, frame, charId, layer, variant = 0, expression = 'normal') {
  const isVal = charId === 'valentina';
  const skin = '#f2c9a0';
  const skinFar = '#d8a878';
  // Neutral, desaturated bases so runtime outfit tints can colorize accurately.
  const shirt = '#d9d9d9';
  const shirtDark = '#a6a6a6';
  const collar = '#f2f2f2';
  const pants = '#bfbfbf';
  const pantsFar = '#999999';
  const shoe = '#3a2a1a';
  const shoeFar = '#2a1d12';
  const knee = '#8c8c8c';
  const kneeFar = '#737373';

  const headX = 13;
  const headY = 4;
  const leadShoulder = { x: 21, y: 12, ex: 24, ey: 14, hx: 26, hy: 14 };
  const trailShoulder = { x: 12, y: 12, ex: 9, ey: 15, hx: 7, hy: 17 };
  const leadLeg = { hipX: 16, hipY: 20, kneeX: 21, kneeY: 19, ankleX: 23, ankleY: 26 };
  const trailLeg = { hipX: 11, hipY: 20, kneeX: 7, kneeY: 22, ankleX: 4, ankleY: 26 };

  if (layer === 'full' || layer === 'base' || layer === 'skin') {
    limb(c, trailShoulder.x, trailShoulder.y, trailShoulder.ex, trailShoulder.ey, 3, skinFar);
    limb(c, trailShoulder.ex, trailShoulder.ey, trailShoulder.hx, trailShoulder.hy, 3, skinFar);
    handShape(c, trailShoulder.hx - 1, trailShoulder.hy, skinFar, '#b8895c');
    limb(c, trailLeg.hipX, trailLeg.hipY, trailLeg.kneeX, trailLeg.kneeY, 4, skinFar);
    limb(c, trailLeg.kneeX, trailLeg.kneeY, trailLeg.ankleX, trailLeg.ankleY, 3, skinFar);
    limb(c, leadLeg.hipX, leadLeg.hipY, leadLeg.kneeX, leadLeg.kneeY, 4, skin);
    limb(c, leadLeg.kneeX, leadLeg.kneeY, leadLeg.ankleX, leadLeg.ankleY, 3, skin);
    c.rect(headX, headY, 10, 7, skin);
    c.rect(headX + 2, headY + 7, 6, 1, skin);
    c.rect(headX + 3, headY + 7, 4, 1, skinFar);
    limb(c, leadShoulder.x, leadShoulder.y, leadShoulder.ex, leadShoulder.ey, 3, skin);
    limb(c, leadShoulder.ex, leadShoulder.ey, leadShoulder.hx, leadShoulder.hy, 3, skin);
    handShape(c, leadShoulder.hx - 1, leadShoulder.hy, skin, skinFar);
  }

  if (layer === 'full' || layer === 'face') drawFaceAt(c, headX - 11, headY, frame, charId, variant, expression);

  if (layer === 'full' || layer === 'outfit_bottom' || layer === 'pants') {
    c.rect(9, 20, 9, 1, pantsFar);
    limb(c, trailLeg.hipX, trailLeg.hipY, trailLeg.kneeX, trailLeg.kneeY, 5, pantsFar);
    limb(c, trailLeg.kneeX, trailLeg.kneeY, trailLeg.ankleX, trailLeg.ankleY, 4, pantsFar);
    c.rect(trailLeg.kneeX - 1, trailLeg.kneeY, 2, 1, kneeFar);
    limb(c, leadLeg.hipX, leadLeg.hipY, leadLeg.kneeX, leadLeg.kneeY, 5, pants);
    limb(c, leadLeg.kneeX, leadLeg.kneeY, leadLeg.ankleX, leadLeg.ankleY, 4, pants);
    c.rect(leadLeg.kneeX - 1, leadLeg.kneeY, 3, 1, knee);
  }

  if (layer === 'full' || layer === 'shoes') {
    shoeShape(c, trailLeg.ankleX - 2, trailLeg.ankleY, 5, shoeFar, '#1d140d', 0);
    shoeShape(c, leadLeg.ankleX - 2, leadLeg.ankleY, 6, shoe, '#21160f', 1);
  }

  if (layer === 'full' || layer === 'outfit_top' || layer === 'shirt') {
    limb(c, trailShoulder.x, trailShoulder.y, trailShoulder.ex, trailShoulder.ey, 4, shirtDark);
    c.rect(trailShoulder.ex - 1, trailShoulder.ey, 3, 1, shirtDark);
    tapered(c, 12, 11, 10, 9, 20, 9, shirt);
    c.rect(12, 11, 10, 1, collar);
    c.rect(12, 12, 1, 7, shirtDark);
    c.rect(9, 19, 9, 1, shirtDark);
    // Wind-blown hem behind the torso.
    c.rect(7, 18, 3, 2, shirtDark);
    c.rect(6, 20, 2, 1, shirtDark);
    limb(c, leadShoulder.x, leadShoulder.y, leadShoulder.ex, leadShoulder.ey, 4, shirt);
    c.rect(leadShoulder.ex - 1, leadShoulder.ey, 3, 1, shirtDark);
  }
  // Body clothing details (separately tintable for customization/equipment).
  if (layer === 'full' || layer === 'outfit_top_detail1') {
    c.rect(12, 15, 8, 2, '#d9d9d9'); // chest stripe
  }
  if (layer === 'full' || layer === 'outfit_top_detail2') {
    c.rect(15, 11, 4, 1, '#e6e6e6'); // collar detail
    c.rect(trailShoulder.ex - 1, trailShoulder.ey, 3, 1, '#d9d9d9');
    c.rect(leadShoulder.ex - 1, leadShoulder.ey, 3, 1, '#d9d9d9');
  }

  if (layer === 'full' || layer === 'hair') drawHairAt(c, headX - 11, headY, frame, charId, variant, false);
}

function drawLowLayer(c, frame, charId, layer, variant = 0, ch = lowCanonicalHeight(frame), expression = 'normal') {
  const isVal = charId === 'valentina';
  const crawl = frame.startsWith('crawl');
  const skin = '#f2c9a0';
  const skinFar = '#d8a878';
  // Neutral, desaturated bases so runtime outfit tints can colorize accurately.
  const shirt = '#d9d9d9';
  const shirtDark = '#a6a6a6';
  const collar = '#f2f2f2';
  const pants = '#bfbfbf';
  const pantsFar = '#999999';
  const shoe = '#3a2a1a';
  const shoeFar = '#2a1d12';
  const knee = '#8c8c8c';
  const kneeFar = '#737373';
  const gy = ch - 2; // top row of ground contact (shoes/hands rest on it)

  // Crouch: low side-view kneel facing right — roughly half standing height.
  // The body lies in a short horizontal wedge; the near leg folds in front
  // with the knee raised over the torso (punched through the shirt layer), and
  // the far leg kneels behind it. Both hands reach forward past the body.
  if (!crawl) {
    const headX = 27;
    const headY = 0;
    // Torso rises up to half the head's height (head is 7 rows: 0-6), so the
    // tucked body covers the lower half of the head.
    const bodyY = 4;
    const bodyH = 5;
    const waistY = bodyY + bodyH; // 9
    // The near knee crosses the bottom row of the shirt and is drawn above it.
    const nearKnee = [30, waistY - 1, 3, 2];
    const drawNearKnee = () => {
      c.rect(...nearKnee, pants);
      c.rect(nearKnee[0], nearKnee[1], nearKnee[2], 1, knee);
    };

    if (layer === 'full' || layer === 'base' || layer === 'skin') {
      // Head and chin shadow (drawn first; torso overlaps the lower half).
      c.rect(headX, headY, 10, 7, skin);
      c.rect(headX + 2, headY + 7, 4, 1, skin);
      c.rect(headX + 5, headY + 7, 3, 1, skinFar);
      // Near arm hangs forward with the hand ahead of the body.
      c.rect(33, 7, 2, 2, skin);
      c.rect(34, 9, 3, 1, skin);
      handShape(c, 37, 9, skin, skinFar);
      // Far arm, slightly higher and behind the near one.
      c.rect(31, 7, 2, 1, skinFar);
      c.rect(33, 8, 3, 1, skinFar);
      handShape(c, 36, 8, skinFar, '#b8895c');
      // Near leg skin (clothed below), extended down to the shoes.
      c.rect(26, waistY, 4, gy - waistY, skin);
      // Far kneeling leg skin, extended down to the shoes.
      c.rect(22, waistY, 3, gy - waistY, skinFar);
    }

    if (layer === 'full' || layer === 'face') drawFaceAt(c, headX - 11, headY, frame, charId, variant, expression);

    if (layer === 'full' || layer === 'outfit_bottom' || layer === 'pants') {
      // Waistband under the torso.
      c.rect(21, waistY, 12, 1, pants);
      // Far leg kneels: thigh, shin, and knee down to the shoes.
      c.rect(22, waistY, 4, 1, pantsFar);
      c.rect(20, waistY + 1, 5, gy - waistY - 1, pantsFar);
      c.rect(18, waistY + 1, 3, Math.max(1, gy - waistY - 2), kneeFar);
      // Near leg folds in front, knee raised, shin down to the shoes.
      c.rect(26, waistY, 4, 1, pants);
      drawNearKnee();
      c.rect(30, waistY + 1, 3, gy - waistY - 1, pants);
    }

    if (layer === 'full' || layer === 'shoes') {
      shoeShape(c, 16, gy, 5, shoeFar, '#1d140d', 0);
      shoeShape(c, 30, gy, 6, shoe, '#21160f', 1);
    }

    if (layer === 'full' || layer === 'outfit_top' || layer === 'shirt') {
      // Flat torso reaching up to half the head height, drawn behind the head.
      c.rect(23, bodyY, 11, 1, shirt);
      c.rect(22, bodyY + 1, 12, 1, shirt);
      c.rect(22, bodyY + 2, 12, 1, shirt);
      c.rect(22, bodyY + 3, 12, 1, shirt);
      c.rect(21, bodyY + 4, 13, 1, shirt);
      c.rect(31, bodyY, 3, 1, collar);
      c.rect(21, bodyY + 4, 6, 1, shirtDark);
      // Keep the head in front of the torso where they overlap.
      if (layer === 'full') {
        c.rect(headX, headY, 10, 7, skin);
        c.rect(headX + 2, headY + 7, 4, 1, skin);
        c.rect(headX + 5, headY + 7, 3, 1, skinFar);
      } else {
        c.clear(headX, headY, 10, 7);
      }
      // Punch the near knee through this layer so it reads as being over the
      // body when the runtime composites the layers.
      if (layer !== 'full') c.clear(...nearKnee);
      // Near arm stays in front of the legs and body.
      c.rect(33, 7, 2, 2, skin);
      c.rect(34, 9, 3, 1, skin);
      handShape(c, 37, 9, skin, skinFar);
    }
    // Body clothing details (separately tintable for customization/equipment).
    if (layer === 'full' || layer === 'outfit_top_detail1') {
      c.rect(22, bodyY + 2, 11, 1, '#d9d9d9'); // chest stripe
    }
    if (layer === 'full' || layer === 'outfit_top_detail2') {
      c.rect(30, bodyY, 4, 1, '#e6e6e6'); // collar detail
    }

    if (layer === 'full') drawNearKnee();
    if (layer === 'full' || layer === 'face') drawFaceAt(c, headX - 11, headY, frame, charId, variant, expression);
    if (layer === 'full' || layer === 'hair') drawHairAt(c, headX - 11, headY, frame, charId, variant, true);
    return;
  }

  // Crawl: hands-and-knees, low and horizontal — roughly half standing height.
  // The torso is a compact square block about half the head's height sitting
  // behind the skull, never reaching back past the pants line. The chest runs
  // under the jaw so the head is set into the body; pants stay below the torso.
  // The diagonal gait pairs the forward hand with the opposite knee.
  const crawlGait = {
    crawl_0: { nearArm: 1, farArm: 0, nearKnee: -1, farKnee: 1 },
    crawl_1: { nearArm: 0, farArm: 0, nearKnee: 0, farKnee: 0 },
    crawl_2: { nearArm: -1, farArm: 1, nearKnee: 1, farKnee: -1 },
    crawl_3: { nearArm: 0, farArm: 0, nearKnee: 0, farKnee: 0 },
  }[frame] || { nearArm: 0, farArm: 0, nearKnee: 0, farKnee: 0 };
  const headX = 28;
  const headY = 0;
  const handY = gy - 1;
  const farHandX = 35 + crawlGait.farArm;
  const nearHandX = 39 + crawlGait.nearArm;
  const farKneeX = 22 + crawlGait.farKnee;
  const nearKneeX = 27 + crawlGait.nearKnee;

  if (layer === 'full' || layer === 'base' || layer === 'skin') {
    // Far arm: shoulder under the chest, forearm planted behind the near hand.
    limb(c, 30, 7, 33, 9, 3, skinFar);
    limb(c, 33, 9, farHandX, handY - 1, 2, skinFar);
    handShape(c, farHandX, handY, skinFar, '#b8895c');
    // Far leg (behind, shadowed).
    c.rect(farKneeX - 1, 10, 3, 2, skinFar);
    // Near leg.
    c.rect(nearKneeX - 1, 10, 3, 2, skin);
    // Head with a shaded jaw underneath.
    c.rect(headX, headY, 10, 7, skin);
    c.rect(headX, headY + 7, 10, 1, skinFar);
    // Near arm in front of the chest, hand planted ahead of the head.
    limb(c, 33, 7, 36, 9, 3, skin);
    limb(c, 36, 9, nearHandX, handY - 1, 2, skin);
    handShape(c, nearHandX, handY, skin, skinFar);
  }

  if (layer === 'full' || layer === 'face') drawFaceAt(c, headX - 11, headY, frame, charId, variant, expression);

  if (layer === 'full' || layer === 'outfit_bottom' || layer === 'pants') {
    // Waist band directly under the torso; legs fold below it only.
    c.rect(20, 9, 13, 1, pants);
    c.rect(20, 9, 4, 1, pantsFar);
    // Far leg: short thigh with the knee planted under the body.
    c.rect(farKneeX - 1, 10, 4, 1, pantsFar);
    c.rect(farKneeX - 2, 11, 4, 1, pantsFar);
    c.rect(farKneeX - 2, 11, 3, 1, kneeFar);
    // Near leg.
    c.rect(nearKneeX - 1, 10, 4, 1, pants);
    c.rect(nearKneeX - 2, 11, 4, 1, pants);
    c.rect(nearKneeX - 2, 11, 3, 1, knee);
  }

  if (layer === 'full' || layer === 'shoes') {
    shoeShape(c, farKneeX - 2, gy, 5, shoeFar, '#1d140d', 0);
    shoeShape(c, nearKneeX - 2, gy, 5, shoe, '#21160f', 0);
  }

  if (layer === 'full' || layer === 'outfit_top' || layer === 'shirt') {
    // Compact square torso, half the head's height, aligned with the pants line
    // at the back. The chest row runs forward under the jaw.
    c.rect(20, 4, 8, 1, shirt);
    c.rect(20, 5, 8, 1, shirt);
    c.rect(20, 6, 8, 1, shirt);
    c.rect(20, 7, 8, 1, shirt);
    c.rect(20, 8, 17, 1, shirt);
    // Collar at the shoulders and a shaded underside at the waist.
    c.rect(26, 4, 2, 1, collar);
    c.rect(20, 7, 8, 1, shirtDark);
  }

  if (layer === 'full' || layer === 'hair') drawHairAt(c, headX - 11, headY, frame, charId, variant, true);
}

function drawHumanoidLayer(c, ox, frame, charId, layer, variant = 0, expression = 'normal') {
  const pose = poseFromFrame(frame);
  const dead = frame === 'dying_0' || frame === 'dead_0';
  const isVal = charId === 'valentina';
  const low = !!(pose.crouch || pose.crawl);
  const skinBase = '#f2c9a0';
  const skin = dead ? '#d8d0e8' : skinBase;
  // Neutral, desaturated bases so runtime outfit tints can colorize accurately.
  const shirt = '#d9d9d9';
  const pants = '#bfbfbf';
  const shoe = '#3a2a1a';

  // Death: character sits on the ground (no hurt/damage body sprite).
  if (dead) {
    const sitHeadX = 10;
    const sitHeadY = 13;
    const sitBodyY = 21;
    const sitBodyH = 6;
    if (layer === 'full' || layer === 'base' || layer === 'skin') {
      c.rect(ox + sitHeadX, sitHeadY, 10, 7, skin);
      c.rect(ox + sitHeadX + 2, sitHeadY + 7, 6, 1, skin);
      c.rect(ox + 8, sitBodyY, 2, 4, skin);   // left arm resting
      c.rect(ox + 22, sitBodyY, 2, 4, skin);  // right arm resting
      c.rect(ox + 10, 28, 10, 2, skin);       // legs/feet area base
    }
    if (layer === 'full' || layer === 'face') {
      drawFaceAt(c, ox, sitHeadY, frame, charId, variant, expression);
    }
    if (layer === 'full' || layer === 'hair') {
      drawHairAt(c, ox, sitHeadY, frame, charId, variant, false);
    }
    if (layer === 'full' || layer === 'outfit_top' || layer === 'shirt') {
      c.rect(ox + 10, sitBodyY, 12, sitBodyH, shirt);
      c.rect(ox + 10, sitBodyY, 12, 1, '#f2f2f2');
      c.rect(ox + 8, sitBodyY, 2, 3, shirt);
      c.rect(ox + 22, sitBodyY, 2, 3, shirt);
    }
    // Body clothing details (separately tintable for customization/equipment).
    if (layer === 'full' || layer === 'outfit_top_detail1') {
      c.rect(ox + 10, sitBodyY + 3, 12, 1, '#d9d9d9');
    }
    if (layer === 'full' || layer === 'outfit_top_detail2') {
      c.rect(ox + 13, sitBodyY, 6, 1, '#e6e6e6');
    }
    if (layer === 'full' || layer === 'outfit_bottom' || layer === 'pants') {
      c.rect(ox + 10, 26, 12, 3, pants); // hips seated
      c.rect(ox + 12, 28, 8, 2, pants);  // legs forward on ground
    }
    if (layer === 'full' || layer === 'shoes') {
      c.rect(ox + 18, 29, 5, 2, shoe);
      c.rect(ox + 11, 29, 5, 2, shoe);
    }
    return;
  }

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
  const legH = lowState ? 3 : 6;
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
    drawFaceAt(c, ox + lean, headY, frame, charId, variant, expression);
  });
  draw(layer === 'full' || layer === 'hair', () => {
    drawHairAt(c, ox + lean, headY, frame, charId, variant, low);
  });
  draw(layer === 'full' || layer === 'outfit_top' || layer === 'shirt', () => {
    const tx = ox + lean;
    c.rect(tx + 10, bodyY, 12, bodyH, shirt);
    c.rect(tx + 10, bodyY, 12, 1, '#f2f2f2');
    // sleeves follow arm pose (same logic as standing, only position changes in low states)
    if (pose.armUpL) c.rect(tx + 8, bodyY - 1, 3, 3, shirt);
    else c.rect(tx + 8, bodyY + 1, 3, 2, shirt);
    if (pose.armUpR) c.rect(tx + 21, bodyY - 1, 3, 3, shirt);
    else if (pose.armOutR) c.rect(tx + 21, bodyY + 2, 3, 2, shirt);
    else c.rect(tx + 21, bodyY + 1, 3, 2, shirt);
  });
  // Body clothing details (separately tintable for customization/equipment).
  if (layer === 'full' || layer === 'outfit_top_detail1') {
    const tx = ox + lean;
    c.rect(tx + 10, bodyY + Math.max(1, Math.floor(bodyH / 2)), 12, 2, '#d9d9d9'); // chest stripe
  }
  if (layer === 'full' || layer === 'outfit_top_detail2') {
    const tx = ox + lean;
    c.rect(tx + 13, bodyY, 6, 1, '#e6e6e6'); // collar detail
  }
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

function buildFrameCanvas(entity, frame, layer, variant, fw, fh, expression = 'normal') {
  const id = entity.ENTITY_ID || entity.id;

  // Low states (crouch/crawl): drawn natively in canonical low space and scaled.
  if (typeof frame === 'string' && (frame.startsWith('crouch') || frame.startsWith('crawl'))) {
    const ch = lowCanonicalHeight(frame);
    const low = new Canvas(LOW_FRAME_W, ch);
    const view = offsetView(low, ch - LOW_CONTENT_H);
    drawLowLayer(view, frame, id, layer, variant, LOW_CONTENT_H, expression);
    if (fw === LOW_FRAME_W * 2 && fh === ch * 2) return low.scale2x();
    return scaleCanvas(low, fw, fh);
  }

  // Walk/run cycles are authored natively in canonical 32x32 space with a
  // four-frame contact/passing gait.
  if (typeof frame === 'string' && (frame.startsWith('walk_') || frame.startsWith('run_'))) {
    const gait = new Canvas(32, 32);
    drawGaitLayer(gait, frame, id, layer, variant, expression);
    return fw === 64 && fh === 64 ? gait.scale2x() : scaleCanvas(gait, fw, fh);
  }

  // Dash is a single explosive lunge pose.
  if (frame === 'dash_0') {
    const dash = new Canvas(32, 32);
    drawDashLayer(dash, frame, id, layer, variant, expression);
    return fw === 64 && fh === 64 ? dash.scale2x() : scaleCanvas(dash, fw, fh);
  }

  // Draw in canonical 32x32 humanoid space, then adapt to state-specific frame size.
  const src = new Canvas(32, 32);
  drawHumanoidLayer(src, 0, frame, id, layer, variant, expression);

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

function buildMatrixSheet(entity, layer, variant = 0, layout = computeMatrixLayout(entity), expression = 'normal') {
  const c = new Canvas(layout.width, layout.height);
  for (const row of layout.rows) {
    for (const frame of row.frames) {
      const fc = buildFrameCanvas(entity, frame.name, layer, variant, row.frameW, row.frameH, expression);
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
    layers: ['base', 'face', 'hair', 'outfit_bottom', 'shoes', 'outfit_top', 'outfit_top_detail1', 'outfit_top_detail2', 'headgear', 'accessory', 'weapon'],
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
  for (const layer of ['base', 'outfit_top', 'outfit_top_detail1', 'outfit_top_detail2', 'outfit_bottom', 'shoes']) {
    writePng(join(dir, 'layers', `${layer}.png`), buildMatrixSheet(entity, layer, 0, layout));
  }
  writePng(join(dir, 'layers', 'hair.png'), buildMatrixSheet(entity, 'hair', defaultHair, layout));
  writePng(join(dir, 'layers', 'face.png'), buildMatrixSheet(entity, 'face', defaultFace, layout, 'normal'));
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.hair_style; i++) {
    writePng(join(dir, 'layers', `hair_style_${i}.png`), buildMatrixSheet(entity, 'hair', i, layout));
  }
  for (let i = 0; i < CUSTOMIZATION_VARIANTS.face; i++) {
    writePng(join(dir, 'layers', `face_${i}.png`), buildMatrixSheet(entity, 'face', i, layout, 'normal'));
    writePng(join(dir, 'layers', `face_${i}_blink.png`), buildMatrixSheet(entity, 'face', i, layout, 'blink'));
    writePng(join(dir, 'layers', `face_${i}_pain.png`), buildMatrixSheet(entity, 'face', i, layout, 'pain'));
    writePng(join(dir, 'layers', `face_${i}_cry.png`), buildMatrixSheet(entity, 'face', i, layout, 'cry'));
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

  for (const clip of ['idle', 'walk', 'jump_rise', 'attack_1', 'sixseven_a', 'dead']) {
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

  for (const rel of ['sheet.png', 'layers/base.png', 'layers/face.png', 'layers/hair.png', 'layers/outfit_top.png', 'layers/outfit_top_detail1.png', 'layers/outfit_top_detail2.png', 'layers/outfit_bottom.png', 'layers/shoes.png']) {
    const p = join(ROOT, 'assets/entities', id, rel);
    if (!existsSync(p)) { errors.push(`entity ${id}: missing ${rel}`); continue; }
    try {
      const sz = readPngSize(p);
      if (sz.w !== expected.width || sz.h !== expected.height) errors.push(`entity ${id}: ${rel} size ${sz.w}x${sz.h} does not match matrix ${expected.width}x${expected.height}`);
    } catch (e) { errors.push(`entity ${id}: invalid PNG ${rel} (${e.message})`); }
  }
  return errors;
}
