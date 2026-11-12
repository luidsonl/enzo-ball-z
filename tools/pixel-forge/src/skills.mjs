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

function iconJump() {
  // Rabbit (jump): cute bunny head/ears on grass-like bg
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, '#6bcb77');
  c.rect(2, 2, 28, 3, '#ffffff44');
  // ears
  c.rect(10, 4, 4, 10, P.outline); c.rect(18, 4, 4, 10, P.outline);
  c.rect(11, 5, 2, 8, '#f5f5f5');  c.rect(19, 5, 2, 8, '#f5f5f5');
  c.rect(11, 6, 2, 5, '#ffb7c5');  c.rect(19, 6, 2, 5, '#ffb7c5');
  // head
  c.rect(8, 12, 16, 12, P.outline);
  c.rect(9, 13, 14, 10, '#f7f7f7');
  // eyes
  c.rect(12, 16, 2, 2, '#1a1a1a'); c.rect(18, 16, 2, 2, '#1a1a1a');
  c.rect(12, 16, 1, 1, '#ffffff'); c.rect(18, 16, 1, 1, '#ffffff');
  // nose/mouth
  c.rect(15, 19, 2, 1, '#ff9db5');
  c.rect(14, 20, 1, 1, '#d88'); c.rect(17, 20, 1, 1, '#d88');
  // cheeks
  c.rect(10, 19, 2, 1, '#ffd1dc'); c.rect(20, 19, 2, 1, '#ffd1dc');
  // little hop lines
  c.rect(6, 25, 4, 1, '#ffffff66'); c.rect(22, 25, 4, 1, '#ffffff66');
  c.rect(7, 27, 3, 1, '#ffffff44'); c.rect(22, 27, 3, 1, '#ffffff44');
  return c;
}

function iconDash() {
  // Running figure (dash)
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, '#3a86ff');
  c.rect(2, 2, 28, 3, '#ffffff44');
  // speed lines
  c.rect(3, 12, 6, 2, '#ffffff88'); c.rect(2, 16, 5, 2, '#ffffff66'); c.rect(4, 20, 4, 2, '#ffffff55');
  // head
  c.rect(14, 6, 5, 5, P.outline); c.rect(15, 7, 3, 3, '#ffd9b3');
  // torso (leaning forward)
  c.rect(13, 11, 5, 7, P.outline); c.rect(14, 12, 3, 5, '#ffe066');
  // front arm
  c.rect(18, 11, 4, 2, P.outline); c.rect(19, 11, 2, 2, '#ffd9b3');
  c.rect(21, 13, 2, 2, '#ffd9b3');
  // back arm
  c.rect(10, 13, 3, 2, P.outline); c.rect(11, 13, 2, 2, '#ffd9b3');
  // front leg
  c.rect(16, 17, 3, 3, P.outline); c.rect(17, 17, 2, 3, '#2f3f8f');
  c.rect(18, 20, 4, 2, P.outline); c.rect(19, 20, 3, 2, '#2f3f8f');
  c.rect(22, 21, 2, 2, '#444');
  // back leg
  c.rect(11, 17, 3, 3, P.outline); c.rect(12, 17, 2, 3, '#2f3f8f');
  c.rect(8, 19, 4, 2, P.outline); c.rect(8, 19, 3, 2, '#2f3f8f');
  c.rect(6, 20, 2, 2, '#444');
  // motion dust
  c.rect(24, 23, 2, 2, '#ffffff55'); c.rect(27, 22, 2, 2, '#ffffff44');
  return c;
}

function iconMeleePunch() {
  // Punch / fist (soco)
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, '#ff6b6b');
  c.rect(2, 2, 28, 3, '#ffffff44');
  // impact burst
  c.rect(23, 8, 3, 3, '#fff59d'); c.rect(26, 12, 3, 2, '#fff59d');
  c.rect(24, 14, 2, 2, '#ffffffaa');
  // arm
  c.rect(4, 15, 10, 6, P.outline); c.rect(5, 16, 9, 4, '#ffd9b3');
  c.rect(3, 17, 2, 2, '#f2c9a0');
  // fist
  c.rect(13, 12, 10, 10, P.outline);
  c.rect(14, 13, 8, 8, '#ffd9b3');
  // knuckles/fingers
  c.rect(21, 13, 3, 2, '#f2c9a0'); c.rect(22, 15, 3, 2, '#f2c9a0');
  c.rect(22, 17, 3, 2, '#f2c9a0'); c.rect(21, 19, 3, 2, '#f2c9a0');
  c.rect(16, 14, 2, 1, '#e8b98a'); c.rect(16, 16, 2, 1, '#e8b98a');
  c.rect(16, 18, 2, 1, '#e8b98a');
  // thumb
  c.rect(13, 19, 3, 3, '#ffd9b3'); c.rect(13, 19, 3, 3, P.outline);
  c.rect(14, 20, 1, 1, '#ffd9b3');
  // motion
  c.rect(25, 16, 4, 2, '#ffffff77'); c.rect(26, 19, 3, 1, '#ffffff55');
  return c;
}

function iconMoggingMoai() {
  // Easter Island moai statue in profile with a prominent jaw (mogging)
  const c = new Canvas(32, 32);
  c.rect(0, 0, 32, 32, P.outline);
  c.rect(1, 1, 30, 30, '#2f3f8f');
  c.rect(2, 2, 28, 3, '#ffffff33');
  // ground
  c.rect(4, 27, 24, 3, '#5b7c3a');
  c.rect(4, 27, 24, 1, '#7fa84f');

  // main head/body block (profile facing right)
  c.rect(9, 5, 10, 22, P.outline);
  c.rect(10, 6, 8, 20, '#9aa0a6');
  c.rect(10, 6, 6, 20, '#b0b7bf');
  c.rect(10, 7, 4, 18, '#c2c8cf');
  // back of head
  c.rect(8, 7, 2, 18, '#8e959d');
  c.rect(8, 7, 2, 18, P.outline);

  // heavy brow ridge
  c.rect(14, 9, 5, 3, '#7d848c');
  c.rect(14, 10, 4, 1, '#6b727a');
  c.rect(14, 9, 5, 3, P.outline);

  // eye socket (profile)
  c.rect(15, 12, 2, 2, '#4a4f55');
  c.rect(16, 12, 1, 1, '#2f3338');

  // long nose (profile, pointing right)
  c.rect(17, 11, 2, 7, '#8e959d');
  c.rect(19, 12, 1, 5, '#7d848c');
  c.rect(17, 17, 3, 2, '#7d848c');
  c.rect(17, 11, 3, 8, P.outline);

  // upper lip area
  c.rect(16, 19, 4, 1, '#7d848c');

  // prominent protruding jaw/chin (maxilar avantajado)
  c.rect(16, 20, 8, 5, P.outline);
  c.rect(17, 21, 6, 3, '#a8afb7');
  c.rect(18, 20, 5, 1, '#b7bec6');
  c.rect(21, 21, 2, 3, '#c2c8cf'); // forward jut
  c.rect(17, 23, 5, 1, '#8e959d');
  // mouth line under nose above jaw
  c.rect(18, 19, 4, 1, '#5a5f66');

  // ear behind head
  c.rect(7, 12, 2, 5, '#9aa0a6');
  c.rect(7, 12, 2, 5, P.outline);

  // neck/base shadow
  c.rect(11, 24, 6, 2, '#8e959d');

  // mogging aura hint
  c.rect(22, 8, 1, 7, '#5ef2ff44');
  c.rect(24, 10, 1, 5, '#5ef2ff33');
  return c;
}

function skillIcon(skillId, hint) {
  if (skillId === 'Jump') return iconJump();
  if (skillId === 'Dash') return iconDash();
  if (skillId === 'Melee') return iconMeleePunch();
  if (skillId === 'Mogging') return iconMoggingMoai();
  return icon(...(hint.icon || [P.stoneDark, P.stoneLight]));
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
      if (kind === 'icon') canvas = skillIcon(skill.id, hint);
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
