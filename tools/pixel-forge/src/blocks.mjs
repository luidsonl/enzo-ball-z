import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
// New-system modular block assets. Legacy assets/sprites/* is not used.
export const GENERATED_DIR = join(ROOT, 'assets/blocks/generated');
export const RUNTIME_TILES_PATH = join(ROOT, 'assets/blocks/tiles.png');
export const MANIFEST_PATH = join(ROOT, 'assets/blocks/tiles.manifest.json');
export const CORE_BLOCKS_PATH = join(ROOT, 'assets/definitions/generated/blocks.json');

export const TILE_CELL = 16;

// Visual/render specs keyed by authoritative block id (contract comes from core export).
const RENDER_SPECS = {
  empty: { sheet: 'tiles', frames: [] },
  ground: { sheet: 'tiles', frames: [0, 1], note: 'top/fill selected by renderer' },
  block: { sheet: 'tiles', frames: [2] },
  platform: { sheet: 'tiles', frames: [3] },
  crate: {
    sheet: 'tiles',
    states: { intact: 4, damaged_1: 5, damaged_2: 6, broken: null },
    note: 'broken removes the tile (empty); state chosen from block damage',
  },
  flag_bottom: { sheet: 'tiles', frames: [7], note: '2-tile goal flag marker (bottom/pole)' },
  flag_top: { sheet: 'tiles', frames: [8], note: '2-tile goal flag marker (top/banner)' },
  breakable_stone: { sheet: 'tiles', states: { intact: 9, damaged_1: 10, damaged_2: 11, broken: null }, note: 'destructible stone; state chosen from block damage' },
};

function fallbackContract() {
  return {
    source: 'fallback',
    blocks: [
      { id: 'empty', symbol: '.', collision: 'passable', resistance: 'None', special: 'none', render: 'empty' },
      { id: 'ground', symbol: '#', collision: 'solid', resistance: 'Unbreakable', special: 'none', render: 'ground' },
      { id: 'block', symbol: 'B', collision: 'solid', resistance: 'Unbreakable', special: 'none', render: 'block' },
      { id: 'platform', symbol: '=', collision: 'platform', resistance: 'None', special: 'none', render: 'platform' },
      { id: 'crate', symbol: 'D', collision: 'solid', resistance: 'Hits(3)', special: 'destructible', render: 'crate' },
    ],
  };
}

export function loadCoreBlockContract() {
  try {
    if (existsSync(CORE_BLOCKS_PATH)) return JSON.parse(readFileSync(CORE_BLOCKS_PATH, 'utf8'));
  } catch {}
  return fallbackContract();
}

export function getBlockDefs() {
  const contract = loadCoreBlockContract();
  return contract.blocks.map((b) => ({
    ...b,
    core: { collision: b.collision, resistance: b.resistance, special: b.special },
    render: RENDER_SPECS[b.id] || { sheet: 'tiles', frames: [] },
  }));
}

export const BLOCK_DEFS = getBlockDefs();

export const TILE_FRAMES = {
  GROUND_TOP: 0,
  GROUND_FILL: 1,
  BLOCK: 2,
  PLATFORM: 3,
  CRATE: 4,
  CRATE_DAMAGED_1: 5,
  CRATE_DAMAGED_2: 6,
  FLAG_BOTTOM: 7,
  FLAG_TOP: 8,
  BREAKABLE_STONE: 9,
  BREAKABLE_STONE_DAMAGED_1: 10,
  BREAKABLE_STONE_DAMAGED_2: 11,
};

function drawGroundTop(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.dirt);
  canvas.rect(ox, 3, 16, 13, P.dirt);
  canvas.rect(ox, 0, 16, 2, P.grassLight);
  canvas.rect(ox, 2, 16, 2, P.grass);
  canvas.rect(ox, 4, 16, 1, P.grassDark);
  [1, 4, 7, 10, 13].forEach((x, i) => {
    canvas.set(ox + x, 0, P.grassLight);
    if (i % 2 === 0) canvas.set(ox + x, -1, P.grassLight);
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
function drawCrate(canvas, ox, damage = 0) {
  canvas.rect(ox, 0, 16, 16, P.crateEdge);
  canvas.rect(ox + 1, 1, 14, 14, P.crate);
  canvas.rect(ox + 1, 1, 14, 2, P.crateLight);
  canvas.rect(ox + 1, 1, 2, 14, P.crateLight);
  canvas.rect(ox + 1, 13, 14, 2, P.crateDark);
  canvas.rect(ox + 13, 1, 2, 14, P.crateDark);
  canvas.rect(ox + 2, 7, 12, 2, P.crateDark);
  canvas.rect(ox + 7, 2, 2, 12, P.crateDark);
  canvas.rect(ox + 2, 7, 12, 1, P.crateEdge);
  canvas.rect(ox + 7, 2, 1, 12, P.crateEdge);
  for (const [x, y] of [[3, 3], [12, 3], [3, 12], [12, 12], [7, 7], [8, 8]]) canvas.set(ox + x, y, P.gold);
  canvas.rect(ox + 4, 4, 2, 1, P.crateLight);
  canvas.rect(ox + 10, 10, 2, 1, P.crateDark);
  const cracks = [
    [[4, 4], [6, 6], [7, 5]],
    [[11, 4], [10, 6], [9, 8]],
    [[4, 11], [6, 10], [8, 9]],
    [[11, 11], [10, 10], [9, 11]],
    [[8, 4], [8, 6]],
    [[5, 8], [7, 8]],
  ];
  if (damage >= 1)
    for (let i = 0; i < Math.min(damage * 2, cracks.length); i++)
      for (const [x, y] of cracks[i]) canvas.set(ox + x, y, i % 2 ? P.crackLight : P.crack);
  if (damage >= 2) {
    canvas.rect(ox + 5, 5, 2, 2, P.crateDark);
    canvas.set(ox + 10, 5, P.crateEdge);
  }
}


function drawFlagBottom(canvas, ox) {
  // transparent base feel: leave mostly empty, draw pole/base
  canvas.rect(ox + 6, 0, 3, 16, '#8a6a3a');
  canvas.rect(ox + 7, 0, 1, 16, '#b89258');
  canvas.rect(ox + 4, 13, 7, 3, '#6f5533');
  canvas.rect(ox + 5, 14, 5, 1, '#8a6a3a');
}
function drawFlagTop(canvas, ox) {
  canvas.rect(ox + 6, 0, 3, 16, '#8a6a3a');
  canvas.rect(ox + 7, 0, 1, 16, '#b89258');
  // banner
  canvas.rect(ox + 8, 2, 7, 8, P.gold);
  canvas.rect(ox + 9, 3, 5, 6, '#ffd84d');
  canvas.rect(ox + 10, 4, 3, 4, '#fff3a8');
  canvas.set(ox + 14, 3, '#e7b63a');
  canvas.set(ox + 14, 8, '#e7b63a');
}


function drawBreakableStone(canvas, ox, damage = 0) {
  canvas.rect(ox,0,16,16,P.stoneDark);
  canvas.rect(ox+1,1,14,14,P.stone);
  canvas.rect(ox+1,1,14,2,P.stoneLight);
  canvas.rect(ox+1,1,2,14,P.stoneLight);
  canvas.rect(ox+1,13,14,2,P.stoneDark);
  canvas.rect(ox+13,1,2,14,P.stoneDark);
  canvas.rect(ox+4,5,3,1,P.stoneDark); canvas.rect(ox+9,4,1,3,P.stoneDark);
  canvas.rect(ox+6,9,4,1,P.stoneDark); canvas.rect(ox+3,10,2,1,P.stoneLight);
  canvas.rect(ox+10,11,2,1,P.stoneLight);
  const cracks = [[[5,4],[6,6],[7,5]],[[10,5],[9,7],[8,8]],[[4,11],[6,10],[8,9]],[[11,11],[10,10],[9,11]],[[8,4],[8,6]]];
  if (damage >= 1) for (let i=0;i<Math.min(damage*2,cracks.length);i++) for (const [x,y] of cracks[i]) canvas.set(ox+x,y,i%2?P.crackLight:P.crack);
  if (damage >= 2) { canvas.rect(ox+5,6,2,2,P.stoneDark); canvas.set(ox+11,6,P.crack); }
}

export function buildTilesSheet() {
  const canvas = new Canvas(TILE_CELL * 12, TILE_CELL);
  drawGroundTop(canvas, TILE_FRAMES.GROUND_TOP * TILE_CELL);
  drawGroundFill(canvas, TILE_FRAMES.GROUND_FILL * TILE_CELL);
  drawBlock(canvas, TILE_FRAMES.BLOCK * TILE_CELL);
  drawPlatform(canvas, TILE_FRAMES.PLATFORM * TILE_CELL);
  drawCrate(canvas, TILE_FRAMES.CRATE * TILE_CELL, 0);
  drawCrate(canvas, TILE_FRAMES.CRATE_DAMAGED_1 * TILE_CELL, 1);
  drawCrate(canvas, TILE_FRAMES.CRATE_DAMAGED_2 * TILE_CELL, 2);
  drawFlagBottom(canvas, TILE_FRAMES.FLAG_BOTTOM * TILE_CELL);
  drawFlagTop(canvas, TILE_FRAMES.FLAG_TOP * TILE_CELL);
  drawBreakableStone(canvas, TILE_FRAMES.BREAKABLE_STONE * TILE_CELL, 0);
  drawBreakableStone(canvas, TILE_FRAMES.BREAKABLE_STONE_DAMAGED_1 * TILE_CELL, 1);
  drawBreakableStone(canvas, TILE_FRAMES.BREAKABLE_STONE_DAMAGED_2 * TILE_CELL, 2);
  return canvas;
}

export function buildManifest() {
  const contract = loadCoreBlockContract();
  return {
    framework: 'pixel-forge',
    version: 1,
    kind: 'blocks',
    contractSource: contract.source,
    cell: { w: TILE_CELL, h: TILE_CELL },
    sheet: 'assets/blocks/tiles.png',
    frames: TILE_FRAMES,
    blocks: getBlockDefs(),
    generatedAt: 'deterministic',
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
    { id: 'block', files: [['default.png', (c) => drawBlock(c, 0)]] },
    { id: 'platform', files: [['default.png', (c) => drawPlatform(c, 0)]] },
    {
      id: 'crate',
      files: [
        ['intact.png', (c) => drawCrate(c, 0, 0)],
        ['damaged_1.png', (c) => drawCrate(c, 0, 1)],
        ['damaged_2.png', (c) => drawCrate(c, 0, 2)],
      ],
    },
    { id: 'flag_bottom', files: [['default.png', (c) => drawFlagBottom(c, 0)]] },
    { id: 'flag_top', files: [['default.png', (c) => drawFlagTop(c, 0)]] },
    { id: 'breakable_stone', files: [['intact.png', (c) => drawBreakableStone(c, 0, 0)], ['damaged_1.png', (c) => drawBreakableStone(c, 0, 1)], ['damaged_2.png', (c) => drawBreakableStone(c, 0, 2)]] },
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
    writeFileSync(
      join(dir, 'manifest.json'),
      JSON.stringify(
        {
          id: def.id,
          sprites: def.files.map(([f]) => f),
          sheet: 'assets/blocks/tiles.png',
          contract: CORE_BLOCKS_PATH.replace(ROOT + '/', ''),
        },
        null,
        2
      ) + '\n'
    );
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
  return { sheet, png, manifest: buildManifest(), perBlockFiles: perBlock ? writePerBlockSprites() : [] };
}
