// Valentina entity sprite contract (separate entity code).
// Defines per-state frame dimensions/anchors for the matrix spritesheet layout.

export const ENTITY_ID = 'valentina';

// Per-state frame dimensions/anchors (entity-specific).
// Crawling and crouched states must have smaller height and larger width than standing states.
export const STATE_FRAME_SPECS = {
  idle:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  walk:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  run:        { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  // Low states: wider than standing and half the standing frame height
  // (32px vs 64px). Two rows of headroom keep tall hair from clipping.
  // Anchored at the frame bottom so feet/knees contact the foot point.
  crouch:     { w: 108, h: 32, anchor: { x: 54, y: 32 } },
  crawl:      { w: 108, h: 32, anchor: { x: 54, y: 32 } },
  jump:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  jump_rise:  { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  jump_fall:  { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  fly:        { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  swim:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  climb:      { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  dash:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  sixseven_a: { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  sixseven_b: { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  attack_1:   { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  attack_2:   { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  attack_3:   { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  kick:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  cast:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  shout:      { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  dying:      { w: 64, h: 64, anchor: { x: 32, y: 62 } },
  dead:       { w: 64, h: 64, anchor: { x: 32, y: 62 } },
};

export function getStateFrameSpec(state) {
  return STATE_FRAME_SPECS[state] || { w: 64, h: 64, anchor: { x: 32, y: 62 } };
}

export const DEFAULT_HAIR_STYLE = 0; // feminine default
export const DEFAULT_FACE = 0;
