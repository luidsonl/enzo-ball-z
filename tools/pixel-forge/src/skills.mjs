import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');
export const CORE_SKILLS_PATH = join(ROOT, 'assets/definitions/generated/skills.json');

const VISUAL_HINTS = {
  Jump: { icon: [P.grassDark, P.grassLight], particles: 'dust' },
  Dash: { icon: [P.stoneDark, P.stoneLight], particles: 'trail' },
  Fly: { icon: ['#3a7bd5', '#b8f7ff'], particles: 'aura' },
  Melee: { icon: [P.dirtDark, P.gold], particles: 'impact' },
  Mogging: { icon: ['#2f3f8f', '#5ef2ff'], entity: '#8aefff', particles: 'impact' },
  NutellaShot: { icon: [P.crateDark, P.woodLight], entity: P.crate, particles: 'splash' },
  Shout: { icon: ['#5b3a8a', P.white], particles: 'wave' },
};

function loadContract() {
  if (existsSync(CORE_SKILLS_PATH)) return JSON.parse(readFileSync(CORE_SKILLS_PATH, 'utf8'));
  return { source: 'fallback', skills: [] };
}

function writePng(absPath, canvas) {
  mkdirSync(dirname(absPath), { recursive: true });
  writeFileSync(absPath, encodePng(canvas));
}

function icon(bg, fg) {
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, bg);
  c.rect(2, 2, 28, 2, '#ffffff44');
  c.rect(6, 14, 20, 4, fg);
  c.rect(14, 6, 4, 20, fg);
  c.rect(10, 10, 12, 12, '#00000022');
  return c;
}

function projectile(fill) {
  const c = new Canvas(32, 32);
  c.rect(6, 13, 20, 6, P.outline);
  c.rect(7, 14, 18, 4, fill);
  c.rect(22, 12, 6, 8, P.white);
  c.rect(8, 15, 8, 2, '#ffffff66');
  return c;
}

function impact(color) {
  const c = new Canvas(32, 32);
  for (const [x, y, w, h] of [[14,4,4,24],[4,14,24,4],[8,8,4,4],[20,8,4,4],[8,20,4,4],[20,20,4,4]]) c.rect(x,y,w,h,P.outline);
  for (const [x, y, w, h] of [[15,5,2,22],[5,15,22,2],[9,9,2,2],[21,9,2,2],[9,21,2,2],[21,21,2,2]]) c.rect(x,y,w,h,color);
  return c;
}

function dust() {
  const c = new Canvas(32, 32);
  for (const [x, y, col] of [[8,22,P.woodLight],[12,20,P.wood],[16,22,P.woodLight],[20,20,P.wood],[24,22,P.woodLight]]) c.rect(x,y,3,3,col);
  c.rect(10,18,2,2,'#ffffff55'); c.rect(18,18,2,2,'#ffffff55');
  return c;
}

function trail() {
  const c = new Canvas(32, 32);
  for (let i = 0; i < 5; i++) c.rect(4 + i * 5, 14 + (i % 2), 4, 4, i % 2 ? '#ffffffaa' : '#a8d8ff');
  c.rect(24, 13, 4, 6, P.white);
  return c;
}

function aura() {
  const c = new Canvas(32, 32);
  c.rect(10, 8, 12, 16, '#7fd8ff55');
  c.rect(8, 10, 16, 12, '#b8f7ff44');
  c.rect(14, 6, 4, 20, '#ffffff66');
  return c;
}

function splash(color = P.crate) {
  const c = new Canvas(32, 32);
  c.rect(13, 10, 6, 10, color);
  c.rect(8, 14, 16, 4, color);
  c.rect(6, 18, 4, 4, color);
  c.rect(22, 18, 4, 4, color);
  c.rect(14, 8, 4, 3, '#ffffff66');
  return c;
}

function wave() {
  const c = new Canvas(32, 32);
  for (let i = 0; i < 4; i++) c.rect(6 + i * 4, 10 + i, 2, 12 - i * 2, i % 2 ? '#e0b8ff' : '#b088ff');
  c.rect(22, 8, 2, 16, '#ffffff88');
  return c;
}

function particleCanvas(kind, hint) {
  if (kind === 'dust') return dust();
  if (kind === 'trail') return trail();
  if (kind === 'aura') return aura();
  if (kind === 'splash') return splash(typeof hint.entity === 'string' ? hint.entity : P.crate);
  if (kind === 'wave') return wave();
  return impact(hint.impact || P.gold);
}

function assetKindFromPath(path) {
  const name = basename(path, '.png');
  if (name.includes('projectile')) return 'projectile';
  if (name.includes('impact') || name.includes('dust') || name.includes('trail') || name.includes('aura') || name.includes('splash') || name.includes('wave')) return name;
  if (name.includes('icon')) return 'icon';
  return name;
}

export function buildSkillAssets({ force = false } = {}) {
  const contract = loadContract();
  const files = [];
  for (const skill of contract.skills) {
    const hint = VISUAL_HINTS[skill.id] || { icon: [P.stoneDark, P.stoneLight], particles: 'impact' };
    const skillDir = join(ROOT, 'assets/skills', skill.id.replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase());
    // Use authoritative declared paths; keep per-module manifest in skill dir inferred from id.
    const declared = skill.assets || {};
    const manifestAssets = {};
    for (const [slot, relPath] of Object.entries(declared)) {
      if (!relPath) continue;
      const abs = join(ROOT, relPath);
      const kind = assetKindFromPath(relPath);
      let canvas = null;
      if (kind === 'icon') canvas = icon(...(hint.icon || [P.stoneDark, P.stoneLight]));
      else if (kind === 'projectile') canvas = projectile(typeof hint.entity === 'string' ? hint.entity : '#8aefff');
      else canvas = particleCanvas(kind === 'impact' ? 'impact' : kind, hint);
      if (canvas && (force || !existsSync(abs))) writePng(abs, canvas);
      files.push(relPath);
      manifestAssets[slot] = basename(relPath);
    }
    const moduleId = skill.id.replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase();
    const moduleDir = join(ROOT, 'assets/skills', moduleId);
    mkdirSync(moduleDir, { recursive: true });
    writeFileSync(join(moduleDir, 'manifest.json'), JSON.stringify({
      framework: 'pixel-forge',
      id: moduleId,
      coreId: skill.id,
      contract: 'assets/definitions/generated/skills.json',
      category: skill.category,
      source: skill.source,
      activation: skill.activation,
      assets: manifestAssets,
      declaredPaths: declared,
    }, null, 2) + '\n');
  }
  return files;
}

export function validateSkillAssets() {
  const contract = loadContract();
  const errors = [];
  for (const skill of contract.skills) {
    const declared = skill.assets || {};
    if (!declared.icon) errors.push(`${skill.id}: missing icon in core contract`);
    for (const [slot, relPath] of Object.entries(declared)) {
      if (!relPath) continue;
      const abs = join(ROOT, relPath);
      if (!existsSync(abs)) errors.push(`${skill.id}: missing declared asset ${slot} -> ${relPath}`);
      else {
        try {
          const b = readFileSync(abs);
          if (b.length < 8 || b[0] !== 137 || b[1] !== 80) errors.push(`${skill.id}: invalid PNG ${relPath}`);
        } catch { errors.push(`${skill.id}: unreadable ${relPath}`); }
      }
    }
  }
  return errors;
}
