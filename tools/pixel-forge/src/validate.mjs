import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { BLOCK_DEFS, RUNTIME_TILES_PATH, TILE_CELL, buildTilesSheet } from './blocks.mjs';
import { validateSkillAssets } from './skills.mjs';
import { validateItemAssets } from './items.mjs';
import { validateEntityAssets } from './entities.mjs';

export function validateBlockDefs(defs = BLOCK_DEFS) {
  const errors = [];
  const ids = new Set();
  const symbols = new Set();
  for (const def of defs) {
    if (!def.id) errors.push('block missing id');
    if (def.id && ids.has(def.id)) errors.push(`duplicate block id: ${def.id}`);
    if (def.id) ids.add(def.id);
    if (def.symbol && symbols.has(def.symbol)) errors.push(`duplicate block symbol: ${def.symbol}`);
    if (def.symbol) symbols.add(def.symbol);
    if (!def.core?.collision) errors.push(`block ${def.id} missing core.collision`);
  }
  return errors;
}

export function validateGeneratedSheet() {
  const errors = [];
  const sheet = buildTilesSheet();
  if (sheet.w % TILE_CELL !== 0 || sheet.h !== TILE_CELL) {
    errors.push(`invalid tiles sheet size: ${sheet.w}x${sheet.h}, expected Nx${TILE_CELL}`);
  }
  if (sheet.w < TILE_CELL * 7) {
    errors.push(`tiles sheet too small for expected frames: ${sheet.w / TILE_CELL}`);
  }
  return { errors, sheet };
}

export function validateBlocks() {
  const errors = [...validateBlockDefs(), ...validateGeneratedSheet().errors];
  // If runtime tiles exist, ensure they are valid PNG signature (cheap check)
  try {
    const path = existsSync(RUNTIME_TILES_PATH)
      ? RUNTIME_TILES_PATH
      : fileURLToPath(new URL('../../../../assets/blocks/tiles.png', import.meta.url));
    const bytes = readFileSync(path);
    if (bytes.length < 8 || bytes[0] !== 137 || bytes[1] !== 80 || bytes[2] !== 78 || bytes[3] !== 71) {
      errors.push('assets/blocks/tiles.png is not a valid PNG');
    }
  } catch {
    // missing runtime sheet is not an error before first build
  }
  return errors;
}

export function validateSkills() {
  return validateSkillAssets();
}

export function validateItems() {
  return validateItemAssets();
}

export function validateEntities() {
  return validateEntityAssets();
}

export function validateAll() {
  return [...validateBlocks(), ...validateSkills(), ...validateItems(), ...validateEntities()];
}
