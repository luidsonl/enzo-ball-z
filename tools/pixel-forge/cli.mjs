#!/usr/bin/env node
import { BLOCK_DEFS, MANIFEST_PATH, RUNTIME_TILES_PATH, writeOutputs } from './src/blocks.mjs';
import { validateAll, validateBlocks, validateEntities, validateItems, validateSkills } from './src/validate.mjs';
import { generateBlock, generateEntity, generateItem, generateSkill } from './src/generate.mjs';
import { buildSkillAssets } from './src/skills.mjs';
import { buildItemAssets } from './src/items.mjs';
import { buildEntityAssets } from './src/entities.mjs';
import { createStats, formatStats, loadCache, saveCache } from './src/cache.mjs';

function usage() {
  console.log(`pixel-forge

Usage:
  node tools/pixel-forge/cli.mjs build blocks [--no-sync-runtime] [--force]
  node tools/pixel-forge/cli.mjs build skills [--force]
  node tools/pixel-forge/cli.mjs build items [--force]
  node tools/pixel-forge/cli.mjs build entities [--force]
  node tools/pixel-forge/cli.mjs validate [blocks|skills|items|entities|all]
  node tools/pixel-forge/cli.mjs inspect block <id>
  node tools/pixel-forge/cli.mjs generate block <id> [--symbol D] [--hits 3] [--passable]
  node tools/pixel-forge/cli.mjs generate skill <id>
  node tools/pixel-forge/cli.mjs generate item <id> [--kind weapon|potion|passive]
  node tools/pixel-forge/cli.mjs generate entity <id>
`);
}

function printErrors(errors) {
  for (const err of errors) console.error(`- ${err}`);
}

function printStats(stats, extra = '') {
  if (!stats) return;
  const suffix = extra ? ` ${extra}` : '';
  console.log(`stats: ${formatStats(stats)}${suffix}`);
}

function cmdBuildBlocks(rest = []) {
  const syncRuntimeTiles = !rest.includes('--no-sync-runtime');
  const force = rest.includes('--force');
  const cache = loadCache();
  const stats = createStats();
  const { sheet, manifest, perBlockFiles, skipped } = writeOutputs({ syncRuntimeTiles, force, cache, stats });
  saveCache(cache);
  console.log(`built block tilesheet: ${sheet.w}x${sheet.h}`);
  console.log(`manifest: ${manifest}`);
  if (syncRuntimeTiles) console.log(`runtime sheet: ${RUNTIME_TILES_PATH}`);
  console.log(`per-block sprites: ${perBlockFiles.length}`);
  console.log(`blocks: ${BLOCK_DEFS.map((b) => b.id).join(', ')}`);
  printStats(stats, skipped ? '(cache hit)' : '');
}

function cmdBuildSkills(rest = []) {
  const force = rest.includes('--force');
  const cache = loadCache();
  const stats = createStats();
  const files = buildSkillAssets({ force, cache, stats });
  saveCache(cache);
  console.log(`skill assets ready: ${files.length} files`);
  printStats(stats);
}

function cmdBuildItems(rest = []) {
  const force = rest.includes('--force');
  const cache = loadCache();
  const stats = createStats();
  const files = buildItemAssets({ force, cache, stats });
  saveCache(cache);
  console.log(`item assets ready: ${files.length} files`);
  printStats(stats);
}

function cmdBuildEntities(rest = []) {
  const force = rest.includes('--force');
  const cache = loadCache();
  const stats = createStats();
  const files = buildEntityAssets({ force, cache, stats });
  saveCache(cache);
  console.log(`entity assets ready: ${files.length} files`);
  printStats(stats);
}

function cmdValidate(target = 'blocks') {
  let errors = [];
  if (target === 'blocks') errors = validateBlocks();
  else if (target === 'skills') errors = validateSkills();
  else if (target === 'items') errors = validateItems();
  else if (target === 'entities') errors = validateEntities();
  else if (target === 'all') errors = validateAll();
  else {
    console.error(`unknown validate target: ${target}`);
    process.exit(1);
  }
  if (errors.length) {
    printErrors(errors);
    process.exit(1);
  }
  console.log(`validation ok (${target})`);
}

function cmdInspectBlock(id) {
  const def = BLOCK_DEFS.find((b) => b.id === id);
  if (!def) {
    console.error(`unknown block id: ${id}`);
    process.exit(1);
  }
  console.log(JSON.stringify(def, null, 2));
}

function cmdGenerate(kind, id, rest = []) {
  if (!id) { usage(); process.exit(1); }
  let result = null;
  if (kind === 'block') {
    result = generateBlock(id, { symbol: flag(rest, '--symbol', 'D'), hits: Number(flag(rest, '--hits', '3')), passable: rest.includes('--passable') });
  } else if (kind === 'skill') result = generateSkill(id);
  else if (kind === 'item') result = generateItem(id, { kind: flag(rest, '--kind', 'passive') });
  else if (kind === 'entity') result = generateEntity(id);
  else { usage(); process.exit(1); }
  console.log(`scaffolded ${kind} ${result.id}`);
  for (const f of result.created) console.log(`  + ${f}`);
  if (!result.created.length) console.log('  (no files created: outputs already exist)');
  for (const n of result.notes || []) console.log(`  note: ${n}`);
}

function flag(args, name, fallback) {
  const i = args.indexOf(name);
  return i >= 0 && args[i + 1] ? args[i + 1] : fallback;
}

const [, , cmd, sub, ...rest] = process.argv;
if (cmd === 'build' && sub === 'blocks') cmdBuildBlocks(rest);
else if (cmd === 'build' && sub === 'skills') cmdBuildSkills(rest);
else if (cmd === 'build' && sub === 'items') cmdBuildItems(rest);
else if (cmd === 'build' && sub === 'entities') cmdBuildEntities(rest);
else if (cmd === 'validate') cmdValidate(sub ?? 'blocks');
else if (cmd === 'inspect' && sub === 'block') cmdInspectBlock(rest[0]);
else if (cmd === 'generate') cmdGenerate(sub, rest[0], rest.slice(1));
else usage();
