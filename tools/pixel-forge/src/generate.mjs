import { mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

function ensureDir(path) {
  mkdirSync(path, { recursive: true });
}

function writeIfMissing(path, content) {
  if (existsSync(path)) return false;
  ensureDir(dirname(path));
  writeFileSync(path, content);
  return true;
}

function iconCanvas(bg, fg, label) {
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, bg);
  c.rect(2, 2, 28, 2, '#ffffff55');
  c.rect(4, 12, 24, 8, fg);
  // simple glyph pixels from label
  const glyph = (label || '?').slice(0, 2).toUpperCase();
  let x = glyph.length === 1 ? 13 : 10;
  for (const ch of glyph) {
    c.rect(x, 10, 5, 12, P.outline);
    c.rect(x + 1, 11, 3, 10, P.white);
    c.set(x + 2, 15, P.outline);
    if (ch === 'I' || ch === '1') c.rect(x + 2, 12, 1, 8, P.outline);
    if (ch === 'S' || ch === '5') { c.rect(x+1,13,3,1,P.outline); c.rect(x+1,16,3,1,P.outline); c.rect(x+1,19,3,1,P.outline); }
    x += 7;
  }
  return c;
}

function tileCanvas(base, edge, accent) {
  const c = new Canvas(16, 16);
  c.rect(0, 0, 16, 16, edge);
  c.rect(1, 1, 14, 14, base);
  c.rect(1, 1, 14, 2, accent);
  c.rect(1, 1, 2, 14, accent);
  c.rect(1, 13, 14, 2, edge);
  c.rect(13, 1, 2, 14, edge);
  c.rect(4, 6, 8, 1, edge);
  c.rect(7, 4, 2, 8, edge);
  return c;
}

export function generateBlock({ id, symbol = '?', solid = true, hits = 0 }) {
  const modDir = join(ROOT, 'crates/enzo-core/src/map/blocks', id);
  const assetDir = join(ROOT, 'assets/blocks', id);
  const collision = solid ? 'CollisionKind::Solid' : 'CollisionKind::Passable';
  const resistance = hits > 0 ? `Resistance::Hits(${hits})` : (solid ? 'Resistance::Unbreakable' : 'Resistance::None');
  const special = hits > 0 ? 'SpecialKind::Destructible' : 'SpecialKind::None';
  const render = hits > 0 ? 'RenderKind::Crate' : (solid ? 'RenderKind::Block' : 'RenderKind::Empty');
  const sym = symbol === '.' ? "Some('.')" : `Some('${symbol}')`;
  const rust = `use crate::map::block::{\n    BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind,\n};\nuse crate::map::visual::{BlockVisual, BlockVisualState};\n\n// TODO: register in BlockId, BLOCK_DEFS, and blocks/mod.rs\npub const DEF: BlockDef = BlockDef {\n    id: BlockId::Block, // TODO: add BlockId::${toPascal(id)}\n    name: "${id}",\n    symbol: ${sym},\n    collision: ${collision},\n    resistance: ${resistance},\n    special: ${special},\n    render: ${render},\n};\n\npub const VISUAL: BlockVisual = BlockVisual {\n    sheet: "assets/blocks/tiles.png",\n    states: &[BlockVisualState { name: "default", frame: Some(2) }],\n};\n`;
  const created = [];
  if (writeIfMissing(join(modDir, 'mod.rs'), rust)) created.push(`rust: ${modDir}/mod.rs`);
  ensureDir(assetDir);
  const spritePath = join(assetDir, 'sprite.png');
  if (!existsSync(spritePath)) {
    writeFileSync(spritePath, encodePng(tileCanvas(P.stone, P.stoneDark, P.stoneLight)));
    created.push(`asset: ${spritePath}`);
  }
  const manifest = { id, symbol, solid, hits, assets: ['sprite.png'], rustModule: `map::blocks::${id}` };
  const manifestPath = join(assetDir, 'block.json');
  if (!existsSync(manifestPath)) {
    writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
    created.push(`asset: ${manifestPath}`);
  }
  return { id, created, notes: [
    'Register the block in crates/enzo-core/src/map/block.rs (BlockId), map/blocks/mod.rs, and BLOCK_DEFS.',
    'Add parser/render behavior if it uses a new collision/special/render kind.',
  ] };
}

export function generateSkill({ id }) {
  const modDir = join(ROOT, 'crates/enzo-core/src/skills', id);
  const assetDir = join(ROOT, 'assets/skills', id);
  const pascal = toPascal(id);
  const rust = `//! ${pascal} skill module.\n\nuse crate::entities::EntityStore;\nuse crate::events::GameEvent;\nuse crate::entities::{AttributeSet, EquipmentTags};\n\nuse super::{\n    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,\n    SkillId, SkillResult, SkillSource, SkillTiming,\n};\n\npub const REQUIREMENTS: Requirements = Requirements::new(\n    &[EntityType::Playable],\n    1,\n    AttributeSet::ZERO,\n    EquipmentTags::NONE,\n);\n\npub static DEF: SkillDef = SkillDef {\n    id: SkillId::Melee, // TODO: add SkillId::${pascal} and register\n    category: SkillCategory::Attack,\n    source: SkillSource::Innate,\n    activation: Activation::Press,\n    requirements: REQUIREMENTS,\n    aura_cost: 0.0,\n    cooldown: 0.0,\n    timing: SkillTiming::PostPhysics,\n    assets: SkillAssets {\n        icon: "assets/skills/${id}/icon.png",\n        entity: None,\n        particles: None,\n    },\n};\n\n#[derive(Clone, Copy, Debug, Default, PartialEq)]\npub struct State;\n\nimpl State {\n    pub fn tick(&mut self, _dt: f32) {}\n}\n\npub fn activate(\n    _state: &mut State,\n    act: SkillActivation<'_>,\n    _store: &mut EntityStore,\n    events: &mut Vec<GameEvent>,\n) -> SkillResult {\n    if !act.input.pressed { return SkillResult::Idle; }\n    events.push(GameEvent::SkillActivated { player: act.entity_id, slot: act.slot, skill: DEF.id });\n    SkillResult::Started\n}\n`;
  const created = [];
  if (writeIfMissing(join(modDir, 'mod.rs'), rust)) created.push(`rust: ${modDir}/mod.rs`);
  ensureDir(assetDir);
  const iconPath = join(assetDir, 'icon.png');
  if (!existsSync(iconPath)) { writeFileSync(iconPath, encodePng(iconCanvas(P.auraDeep, P.aura, pascal))); created.push(`asset: ${iconPath}`); }
  const manifestPath = join(assetDir, 'skill.json');
  if (!existsSync(manifestPath)) { writeFileSync(manifestPath, JSON.stringify({ id, icon: 'icon.png', assets: ['icon.png'] }, null, 2)+'\n'); created.push(`asset: ${manifestPath}`); }
  return { id, created, notes: ['Register SkillId/DEF in skills registry, mod.rs, and any dispatch/loadout as needed.'] };
}

export function generateItem({ id, kind = 'passive' }) {
  const modDir = join(ROOT, 'crates/enzo-core/src/entities/item', id);
  const assetDir = join(ROOT, 'assets/items', id);
  const pascal = toPascal(id);
  const category = kind === 'weapon' ? 'ItemCategory::Weapon' : kind === 'potion' ? 'ItemCategory::Potion' : 'ItemCategory::Accessory';
  const rust = `//! ${pascal} item module.\n\nuse crate::entities::base::attributes::AttributeSet;\nuse crate::entities::base::equipment::{Equipment, EquipmentTags};\nuse crate::entities::item::{ItemCategory, ItemData, ItemKind, ItemRequirements};\nuse crate::skills::EntityType;\n\n// TODO: add ItemKind::${pascal} and ItemData::${id.toUpperCase()} wiring as needed.\npub const DATA: ItemData = ItemData {\n    kind: ItemKind::Sword, // TODO\n    category: ${category},\n    equipment: Equipment::NONE,\n    requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),\n    heal: 0.0,\n};\n\npub const VISUAL: &str = "assets/items/${id}/icon.png";\n`;
  const created = [];
  if (writeIfMissing(join(modDir, 'mod.rs'), rust)) created.push(`rust: ${modDir}/mod.rs`);
  ensureDir(assetDir);
  const iconPath = join(assetDir, 'icon.png');
  if (!existsSync(iconPath)) { writeFileSync(iconPath, encodePng(iconCanvas(P.wood, P.gold, pascal))); created.push(`asset: ${iconPath}`); }
  const manifestPath = join(assetDir, 'item.json');
  if (!existsSync(manifestPath)) { writeFileSync(manifestPath, JSON.stringify({ id, kind, icon: 'icon.png' }, null, 2)+'\n'); created.push(`asset: ${manifestPath}`); }
  return { id, created, notes: ['Register ItemKind/ItemData in the item system if needed.'] };
}

export function generateEntity({ id }) {
  const modDir = join(ROOT, 'crates/enzo-core/src/entities/actors', id);
  const assetDir = join(ROOT, 'assets/entities', id);
  const pascal = toPascal(id);
  const rust = `//! ${pascal} entity module.\n//!\n//! Define attributes, body/spec constants, animations, and visual metadata here.\n\nuse crate::math::Vec2;\n\npub const SIZE: Vec2 = Vec2::new(24.0, 30.0);\npub const SHEET: &str = "assets/entities/${id}/sheet.png";\npub const CELL: f32 = 32.0;\npub const ANIMATIONS: &[(&str, usize)] = &[("idle", 2), ("run", 4)];\n`;
  const created = [];
  if (writeIfMissing(join(modDir, 'mod.rs'), rust)) created.push(`rust: ${modDir}/mod.rs`);
  ensureDir(assetDir);
  const sheetPath = join(assetDir, 'sheet.png');
  if (!existsSync(sheetPath)) { writeFileSync(sheetPath, encodePng(iconCanvas(P.slimeDark, P.slime, pascal))); created.push(`asset: ${sheetPath}`); }
  const manifestPath = join(assetDir, 'entity.json');
  if (!existsSync(manifestPath)) { writeFileSync(manifestPath, JSON.stringify({ id, sheet: 'sheet.png', cell: 32, animations: [['idle',2],['run',4]] }, null, 2)+'\n'); created.push(`asset: ${manifestPath}`); }
  return { id, created, notes: ['Add specs/controllers/AI/render integration and register module exports as needed.'] };
}

function toPascal(id) {
  return id.split(/[_-]+/).filter(Boolean).map(s => s[0].toUpperCase() + s.slice(1)).join('');
}
