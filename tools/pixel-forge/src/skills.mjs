import { mkdirSync, writeFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Canvas, encodePng } from './png.mjs';
import { P } from './palette.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

function write(path, canvas) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, encodePng(canvas));
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
  for (const [x,y,w,h] of [[14,4,4,24],[4,14,24,4],[8,8,4,4],[20,8,4,4],[8,20,4,4],[20,20,4,4]]) c.rect(x,y,w,h,P.outline);
  for (const [x,y,w,h] of [[15,5,2,22],[5,15,22,2],[9,9,2,2],[21,9,2,2],[9,21,2,2],[21,21,2,2]]) c.rect(x,y,w,h,color);
  return c;
}

export const SKILL_VISUALS = [
  { id: 'jump', icon: [P.grassDark, P.grassLight] },
  { id: 'dash', icon: [P.stoneDark, P.stoneLight] },
  { id: 'melee', icon: [P.dirtDark, P.gold], impact: P.gold },
  { id: 'mogging', icon: ['#2f3f8f', '#5ef2ff'], projectile: '#8aefff', impact: '#c8ffff' },
  { id: 'nutella_shot', icon: [P.crateDark, P.woodLight], projectile: P.crate, impact: P.woodLight },
  { id: 'shout', icon: ['#5b3a8a', P.white], impact: '#e0b8ff' },
  { id: 'fly', icon: ['#3a7bd5', '#b8f7ff'] },
];

export function buildSkillAssets() {
  const files = [];
  for (const skill of SKILL_VISUALS) {
    const dir = join(ROOT, 'assets/skills', skill.id);
    const manifest = { id: skill.id, assets: {} };
    if (skill.icon) { write(join(dir, 'icon.png'), icon(...skill.icon)); manifest.assets.icon = 'icon.png'; files.push(`${skill.id}/icon.png`); }
    if (skill.projectile) { write(join(dir, 'projectile.png'), projectile(skill.projectile)); manifest.assets.projectile = 'projectile.png'; files.push(`${skill.id}/projectile.png`); }
    if (skill.impact) { write(join(dir, 'impact.png'), impact(skill.impact)); manifest.assets.impact = 'impact.png'; files.push(`${skill.id}/impact.png`); }
    writeFileSync(join(dir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  }
  return files;
}
