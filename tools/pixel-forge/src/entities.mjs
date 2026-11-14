// Entity sprite/asset facade.
// - Per-entity code defines per-state frame dimensions/anchors and entity specifics.
// - Humanoids use matrix atlas layout: rows = states, columns = frames of each state.

import * as enzo from './entities/enzo.mjs';
import * as valentina from './entities/valentina.mjs';
import { buildSlimeAssets, validateSlimeAssets } from './entities/slime.mjs';
import {
  HUMANOID_CLIPS,
  HUMANOID_FALLBACKS,
  HUMANOID_FRAMES,
  HUMANOID_STATE_ORDER,
  buildHumanoidEntity,
  computeMatrixLayout,
  validateHumanoidEntity,
} from './entities/humanoid.mjs';

export { HUMANOID_CLIPS, HUMANOID_FALLBACKS, HUMANOID_FRAMES, HUMANOID_STATE_ORDER, computeMatrixLayout };

// Compatibility export: compute layout for a humanoid entity definition (entity owns frame dimensions).
export function computeHumanoidMatrixLayout(entity = enzo) {
  return computeMatrixLayout(entity);
}

const HUMANOID_ENTITIES = [enzo, valentina];

export function buildEntityAssets({ force = false } = {}) {
  const files = [];
  // Humanoids always regenerate rig-aligned assets (deterministic, contract-driven).
  for (const entity of HUMANOID_ENTITIES) {
    files.push(...buildHumanoidEntity(entity));
  }
  files.push(...buildSlimeAssets({ force }));
  return files;
}

export function validateEntityAssets() {
  const errors = [];
  for (const entity of HUMANOID_ENTITIES) {
    validateHumanoidEntity(entity, errors);
  }
  validateSlimeAssets(errors);
  return errors;
}

