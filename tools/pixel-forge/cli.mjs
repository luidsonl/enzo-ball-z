#!/usr/bin/env node
import { BLOCK_DEFS, MANIFEST_PATH, RUNTIME_TILES_PATH, writeOutputs } from './src/blocks.mjs';
import { validateBlocks } from './src/validate.mjs';
import { generateBlock, generateEntity, generateItem, generateSkill } from './src/generate.mjs';

function usage() {
  console.log(`pixel-forge

Usage:
  node tools/pixel-forge/cli.mjs build blocks [--no-sync-runtime]
  node tools/pixel-forge/cli.mjs validate blocks
  node tools/pixel-forge/cli.mjs inspect block <id>
  node tools/pixel-forge/cli.mjs generate block <id> [--symbol D] [--hits 3] [--passable]
  node tools/pixel-forge/cli.mjs generate skill <id>
  node tools/pixel-forge/cli.mjs generate item <id> [--kind weapon|potion|passive]
  node tools/pixel-forge/cli.mjs generate entity <id>
`);
}

function flag(args, name, fallback = undefined) {
  const i = args.indexOf(name);
  if (i === -1) return fallback;
  const v = args[i + 1];
  if (!v || v.startsWith('--')) return true;
  return v;
}

const [, , cmd, sub, ...rest] = process.argv;

if (cmd === 'build' && sub === 'blocks') {
  const syncRuntimeTiles = !rest.includes('--no-sync-runtime');
  const { sheet, manifest, perBlockFiles } = writeOutputs({ syncRuntimeTiles });
  console.log(`built block tiles: ${sheet.w}x${sheet.h}`);
  console.log(`manifest: ${MANIFEST_PATH}`);
  if (syncRuntimeTiles) console.log(`runtime sheet: ${RUNTIME_TILES_PATH}`);
  console.log(`per-module sprites: ${perBlockFiles.length}`);
  console.log(`blocks: ${manifest.blocks.map((b) => b.id).join(', ')}`);
  process.exit(0);
}

if (cmd === 'validate' && sub === 'blocks') {
  const errors = validateBlocks();
  if (errors.length) {
    console.error('pixel-forge: validation failed');
    for (const e of errors) console.error(`- ${e}`);
    process.exit(1);
  }
  console.log('pixel-forge: blocks OK');
  process.exit(0);
}

if (cmd === 'inspect' && sub === 'block') {
  const id = rest[0];
  const def = BLOCK_DEFS.find((b) => b.id === id);
  if (!def) {
    console.error(`unknown block: ${id ?? '(missing id)'}`);
    process.exit(1);
  }
  console.log(JSON.stringify(def, null, 2));
  process.exit(0);
}

if (cmd === 'generate') {
  const kind = sub;
  const id = rest[0];
  if (!id) { console.error('missing id'); usage(); process.exit(1); }
  let result;
  if (kind === 'block') result = generateBlock({ id, symbol: flag(rest, '--symbol', '?'), solid: !rest.includes('--passable'), hits: Number(flag(rest, '--hits', 0)) || 0 });
  else if (kind === 'skill') result = generateSkill({ id });
  else if (kind === 'item') result = generateItem({ id, kind: flag(rest, '--kind', 'passive') });
  else if (kind === 'entity') result = generateEntity({ id });
  else { console.error(`unknown generate kind: ${kind}`); usage(); process.exit(1); }
  console.log(`generated ${kind}: ${result.id}`);
  for (const c of result.created) console.log(`  + ${c}`);
  for (const n of result.notes || []) console.log(`  note: ${n}`);
  process.exit(0);
}

usage();
process.exit(cmd ? 1 : 0);
