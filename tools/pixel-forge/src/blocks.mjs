import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
export const GENERATED_DIR = join(ROOT, 'assets/sprites/generated');
export const RUNTIME_TILES_PATH = join(ROOT, 'assets/sprites/tiles.png');
export const MANIFEST_PATH = join(GENERATED_DIR, 'blocks.manifest.json');

export const TILE_CELL = 16;

/**
 * Block visual definitions. `core` mirrors the authoritative gameplay contract
 * in `enzo-core/src/map.rs` (id/symbol/properties) at a documentation/manifest
 * level; gameplay rules remain in core.
 */
export const BLOCK_DEFS = [
  {
    id: 'ground',
    symbol: '#',
    core: { collision: 'solid', resistance: 'unbreakable', special: 'none' },
    render: { sheet: 'tiles', frames: [0, 1], note: 'top/fill selected by renderer' },
  },
  {
    id: 'block',
    symbol: 'B',
    core: { collision: 'solid', resistance: 'unbreakable', special: 'none' },
    render: { sheet: 'tiles', frames: [2] },
  },
  {
    id: 'platform',
    symbol: '=',
    core: { collision: 'platform', resistance: 'none', special: 'none' },
    render: { sheet: 'tiles', frames: [3] },
  },
  {
    id: 'crate',
    symbol: 'D',
    core: { collision: 'solid', resistance: { hits: 3 }, special: 'destructible' },
    render: {
      sheet: 'tiles',
      states: { intact: 4, damaged_1: 5, damaged_2: 6, broken: null },
      note: 'broken removes the tile (empty); state chosen from block damage',
    },
  },
  {
    id: 'empty',
    symbol: '.',
    core: { collision: 'passable', resistance: 'none', special: 'none' },
    render: { sheet: 'tiles', frames: [] },
  },
];

export const TILE_FRAMES = {
  GROUND_TOP: 0,
  GROUND_FILL: 1,
  BLOCK: 2,
  PLATFORM: 3,
  CRATE: 4,
  CRATE_DAMAGED_1: 5,
  CRATE_DAMAGED_2: 6,
};

function drawGroundTop(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.dirt);
  canvas.rect(ox, 3, 16, 13, P.dirt);
  canvas.rect(ox, 0, 16, 2, P.grassLight);
  canvas.rect(ox, 2, 16, 2, P.grass);
  canvas.rect(ox, 4, 16, 1, P.grassDark);
  const blades = [1, 4, 7, 10, 13];
  blades.forEach((x, index) => {
    canvas.set(ox + x, 0, P.grassLight);
    if (index % 2 === 0) canvas.set(ox + x, -1, P.grassLight);
  });
  canvas.rect(ox + 2, 7, 2, 1, P.dirtLight);
  canvas.rect(ox + 9, 10, 3, 1, P.dirtLight);
  canvas.rect(ox + 12, 6, 1, 2, P.dirtDark);
  canvas.rect(ox + 4, 12, 2, 2, P.dirtDark);
  canvas.set(ox + 7, 9, P.dirtLight);
}

function drawGroundFill(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.dirt);
  canvas.rect(ox, 0, 16, 1, P.dirtDark);
  canvas.rect(ox + 2, 3, 3, 1, P.dirtLight);
  canvas.rect(ox + 9, 5, 2, 2, P.dirtDark);
  canvas.rect(ox + 5, 9, 3, 1, P.dirtLight);
  canvas.rect(ox + 12, 12, 2, 1, P.dirtDark);
  canvas.set(ox + 1, 13, P.dirtLight);
  canvas.set(ox + 8, 14, P.dirtDark);
}

function drawBlock(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.stoneDark);
  canvas.rect(ox + 1, 1, 14, 14, P.stone);
  canvas.rect(ox + 1, 1, 14, 2, P.stoneLight);
  canvas.rect(ox + 1, 1, 2, 14, P.stoneLight);
  canvas.rect(ox + 1, 13, 14, 2, P.stoneDark);
  canvas.rect(ox + 13, 1, 2, 14, P.stoneDark);
  canvas.rect(ox + 4, 5, 2, 1, P.stoneDark);
  canvas.rect(ox + 9, 4, 1, 3, P.stoneDark);
  canvas.rect(ox + 7, 9, 4, 1, P.stoneDark);
  canvas.rect(ox + 5, 10, 1, 2, P.stoneDark);
  canvas.rect(ox + 11, 11, 2, 1, P.stoneLight);
}

function drawPlatform(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.woodDark);
  canvas.rect(ox + 1, 1, 14, 5, P.wood);
  canvas.rect(ox + 1, 7, 14, 7, P.wood);
  canvas.rect(ox + 1, 1, 14, 1, P.woodLight);
  canvas.rect(ox + 1, 7, 14, 1, P.woodLight);
  canvas.rect(ox + 4, 2, 1, 5, P.woodDark);
  canvas.rect(ox + 11, 2, 1, 5, P.woodDark);
  canvas.rect(ox + 7, 8, 1, 6, P.woodDark);
  canvas.set(ox + 2, 3, P.woodDark);
  canvas.set(ox + 13, 9, P.woodDark);
  canvas.set(ox + 13, 4, P.woodLight);
  canvas.set(ox + 3, 11, P.woodLight);
}

function drawCrateBase(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.crateEdge);
  canvas.rect(ox + 1, 1, 14, 14, P.crate);
  canvas.rect(ox + 1, 1, 14, 2, P.crateLight);
  canvas.rect(ox + 1, 1, 2, 14, P.crateLight);
  canvas.rect(ox + 1, 13, 14, 2, P.crateDark);
  canvas.rect(ox + 13, 1, 2, 14, P.crateDark);
  // planks/cross
  canvas.rect(ox + 2, 7, 12, 2, P.crateDark);
  canvas.rect(ox + 7, 2, 2, 12, P.crateDark);
  canvas.rect(ox + 2, 7, 12, 1, P.crateEdge);
  canvas.rect(ox + 7, 2, 1, 12, P.crateEdge);
  // nails/bolts
  for (const [x, y] of [[3,3],[12,3],[3,12],[12,12],[7,7],[8,8]]) canvas.set(ox+x, y, P.gold);
  canvas.rect(ox + 4, 4, 2, 1, P.crateLight);
  canvas.rect(ox + 10, 10, 2, 1, P.crateDark);
}

function drawCracks(canvas, ox, amount) {
  const cracks = [
    [[4,4],[6,6],[7,5]],
    [[11,4],[10,6],[9,8]],
    [[4,11],[6,10],[8,9]],
    [[11,11],[10,10],[9,11]],
    [[8,4],[8,6]],
    [[5,8],[7,8]],
  ];
  for (let i = 0; i < amount && i < cracks.length; i += 1) {
    for (const [x, y] of cracks[i]) canvas.set(ox + x, y, i % 2 ? P.crackLight : P.crack);
  }
}

function drawCrate(canvas, ox, damage = 0) {
  drawCrateBase(canvas, ox);
  if (damage >= 1) drawCracks(canvas, ox, damage * 2);
  if (damage >= 2) {
    canvas.rect(ox + 5, 5, 2, 2, P.crateDark);
    canvas.set(ox + 10, 5, P.crateEdge);
  }
}

export function buildTilesSheet() {
  const frames = 7;
  const canvas = new Canvas(TILE_CELL * frames, TILE_CELL);
  drawGroundTop(canvas, TILE_FRAMES.GROUND_TOP * TILE_CELL);
  drawGroundFill(canvas, TILE_FRAMES.GROUND_FILL * TILE_CELL);
  drawBlock(canvas, TILE_FRAMES.BLOCK * TILE_CELL);
  drawPlatform(canvas, TILE_FRAMES.PLATFORM * TILE_CELL);
  drawCrate(canvas, TILE_FRAMES.CRATE * TILE_CELL, 0);
  drawCrate(canvas, TILE_FRAMES.CRATE_DAMAGED_1 * TILE_CELL, 1);
  drawCrate(canvas, TILE_FRAMES.CRATE_DAMAGED_2 * TILE_CELL, 2);
  return canvas;
}

export function buildManifest() {
  return {
    framework: 'pixel-forge',
    version: 1,
    kind: 'blocks',
    cell: { w: TILE_CELL, h: TILE_CELL },
    sheet: 'assets/sprites/tiles.png',
    frames: TILE_FRAMES,
    blocks: BLOCK_DEFS,
    generatedAt: 'deterministic', // keep output stable when inputs are stable
  };
}

function writePngFile(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
}

export function writePerBlockSprites() {
  const out = [];
  const defs = [
    { id: 'ground', files: [['top.png', (c) => drawGroundTop(c, 0)], ['fill.png', (c) => drawGroundFill(c, 0)]] },
    { id: 'brick', files: [['default.png', (c) => drawBlock(c, 0)]] },
    { id: 'platform', files: [['default.png', (c) => drawPlatform(c, 0)]] },
    { id: 'crate', files: [['intact.png', (c) => drawCrate(c, 0, 0)], ['damaged_1.png', (c) => drawCrate(c, 0, 1)], ['damaged_2.png', (c) => drawCrate(c, 0, 2)]] },
  ];
  for (const def of defs) {
    const dir = join(ROOT, 'assets/blocks', def.id);
    for (const [name, draw] of def.files) {
      const c = new Canvas(TILE_CELL, TILE_CELL);
      draw(c);
      const path = join(dir, name);
      writePngFile(path, c);
      out.push(path);
    }
    const localManifest = { id: def.id, sprites: def.files.map(([f]) => f), sheet: 'assets/sprites/tiles.png' };
    writeFileSync(join(dir, 'manifest.json'), JSON.stringify(localManifest, null, 2) + '\n');
  }
  return out;
}

export function writeOutputs({ syncRuntimeTiles = true, perBlock = true } = {}) {
  mkdirSync(GENERATED_DIR, { recursive: true });
  const sheet = buildTilesSheet();
  const png = encodePng(sheet);
  writeFileSync(join(GENERATED_DIR, 'tiles.blocks.png'), png);
  writeFileSync(MANIFEST_PATH, JSON.stringify(buildManifest(), null, 2) + '\n');
  if (syncRuntimeTiles) {
    mkdirSync(dirname(RUNTIME_TILES_PATH), { recursive: true });
    writeFileSync(RUNTIME_TILES_PATH, png);
  }
  const perBlockFiles = perBlock ? writePerBlockSprites() : [];
  return { sheet, png, manifest: buildManifest(), perBlockFiles };
}
