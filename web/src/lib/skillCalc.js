export function uiLangFromState(state) {
  const lang = state?.settings?.language || "pt-BR";
  return String(lang).toLowerCase().startsWith("en") ? "en" : "pt";
}

export function labelText(state, key, en, pt) {
  const labels = state?.labels || {};
  if (labels[key]) return labels[key];
  return uiLangFromState(state) === "pt" ? pt : en;
}

export function calcUiLabel(state) {
  return labelText(state, "skill_calc", "Calculation", "Cálculo");
}
export function rangeUiLabel(state) {
  return labelText(state, "range", "Range", "Alcance");
}
export function knockbackUiLabel(state) {
  return labelText(state, "knockback", "Knockback", "Repulsão");
}
export function seeCalcUiLabel(state) {
  return labelText(state, "see_calc", "See calc", "Ver cálculo");
}

export function formatDamagePreview(entry, state) {
  const values = entry?.damage_preview || [];
  if (!values.length) return state?.labels?.skill_no_damage || "—";
  const fmt = (v) => (Number.isInteger(v) ? String(v) : Number(v).toFixed(1).replace(/\.0$/, ""));
  return values.map(fmt).join(" / ");
}

export function formatCooldown(sec) {
  if (sec == null) return "—";
  const n = Number(sec);
  if (!Number.isFinite(n) || n <= 0) return "0s";
  return `${n.toFixed(2).replace(/\.00$/, "").replace(/(\.\d)0$/, "$1")}s`;
}

export function formatAuraCost(cost) {
  if (cost == null) return "0";
  const n = Number(cost);
  if (!Number.isFinite(n)) return "0";
  return Number.isInteger(n) ? String(n) : n.toFixed(1).replace(/\.0$/, "");
}

export function translateSkillCalc(text, lang = "en") {
  if (!text || lang !== "pt") return text || "";
  const trLine = (line) => {
    let m;
    if (line === "Damage (physical)") return "Dano (físico)";
    if (line === "Damage") return "Dano";
    if (line === "Range") return "Alcance";
    if (line === "Knockback") return "Repulsão";

    if (line === "- Requires: Nutella Gun (equipment)") return "- Requer: Arma de Nutella (equipamento)";
    if (line === "- Upward lift: yes") return "- Elevação para cima: sim";
    if (line === "- Total: fixed knockback") return "- Total: repulsão fixa";
    if (line === "- Total: none") return "- Total: nenhum";

    m = line.match(/^- Strength: (.+) pts -> base (.+)$/);
    if (m) return `- Força: ${m[1]} pts -> base ${m[2]}`;

    m = line.match(/^- Ego: (.+) pts -> base (.+)$/);
    if (m) return `- Ego: ${m[1]} pts -> base ${m[2]}`;

    m = line.match(/^- Ego: (.+) pts -> \+(.+)$/);
    if (m) return `- Ego: ${m[1]} pts -> +${m[2]}`;

    m = line.match(/^- Skill level: eff\. (.+) \(tier (.+), includes item bonuses\) -> \+(.+)$/);
    if (m) return `- Nível da habilidade: efetivo ${m[1]} (nível ${m[2]}, inclui bônus de itens) -> +${m[3]}`;

    m = line.match(/^- Skill tier (.+) \(skill level (.+), includes item bonuses\): \+(.+) from Ego$/);
    if (m) return `- Nível da habilidade ${m[1]} (nível de habilidade ${m[2]}, inclui bônus de itens): +${m[3]} de Ego`;

    m = line.match(/^- Skill tier (.+) \(skill level (.+), includes item bonuses\)$/);
    if (m) return `- Nível da habilidade ${m[1]} (nível de habilidade ${m[2]}, inclui bônus de itens)`;

    m = line.match(/^- Skill tier (.+) \(skill level (.+), includes item bonuses\): sets projectile travel$/);
    if (m) return `- Nível da habilidade ${m[1]} (nível de habilidade ${m[2]}, inclui bônus de itens): define o alcance do projétil`;

    m = line.match(/^- Base: ~(.+) px$/);
    if (m) return `- Base: ~${m[1]} px`;

    m = line.match(/^- Base: (.+) px$/);
    if (m) return `- Base: ${m[1]} px`;

    m = line.match(/^- Base push: (.+) px\/s$/);
    if (m) return `- Empurrão base: ${m[1]} px/s`;

    m = line.match(/^- Base: (.+)$/);
    if (m) return `- Base: ${m[1]}`;

    m = line.match(/^- Push on hit: (.+) px\/s$/);
    if (m) return `- Empurrão ao acertar: ${m[1]} px/s`;

    m = line.match(/^- Stun: (.+) s$/);
    if (m) return `- Atordoamento: ${m[1]} s`;

    m = line.match(/^- Total: (.+) per hit \((.+) projectile\(s\)\)$/);
    if (m) return `- Total: ${m[1]} por acerto (${m[2]} projétil(is))`;

    m = line.match(/^- Total: ~(.+) px \(speed (.+) px\/s, lifetime (.+) s\)$/);
    if (m) return `- Total: ~${m[1]} px (velocidade ${m[2]} px/s, duração ${m[3]} s)`;

    m = line.match(/^- Total: (.+) px\/s push \+ upward lift$/);
    if (m) return `- Total: empurrão de ${m[1]} px/s + elevação para cima`;

    m = line.match(/^- Total: ~(.+) px$/);
    if (m) return `- Total: ~${m[1]} px`;

    m = line.match(/^- Total: (.+) px$/);
    if (m) return `- Total: ${m[1]} px`;

    m = line.match(/^- Total: (.+)$/);
    if (m) return `- Total: ${m[1]}`;

    return line;
  };

  return String(text).split("\n").map(trLine).join("\n");
}
