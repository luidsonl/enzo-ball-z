<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { UiAction, closeMenuAndFocusGame, equipItem, focusGameCanvas, loadoutAssign, loadoutClear, setPlayableCharacter, setVisualProfileField, uiAction, unequipSlot } from "../../bridge/bridge.js";
import { mobileControlsVisible } from "../../lib/mobile.js";

const props = defineProps({
  state: { type: Object, default: null },
});

const openPanel = ref(null); // status | equipment | skills | items | customize | null
const selectedSkillSlot = ref(0);
const fullscreen = ref(false);
const detailSkill = ref(null); // catalog entry shown in skills detail window
const openSkillCalc = ref(null); // "damage" | "range" | "knockback" | null

const state = computed(() => props.state);
const showHud = computed(() => state.value?.scene === "game");
const player = computed(() => state.value?.player ?? null);
const labels = computed(() => state.value?.labels ?? {});
const phase = computed(() => state.value?.phase || "Playing");

watch(showHud, (v) => { if (!v) openPanel.value = null; });

function isFullscreen() {
  return !!(document.fullscreenElement || document.webkitFullscreenElement);
}
function syncFullscreen() { fullscreen.value = isFullscreen(); }
async function toggleFullscreen() {
  try {
    if (isFullscreen()) {
      if (document.exitFullscreen) await document.exitFullscreen();
      else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
    } else {
      const root = document.documentElement;
      if (root.requestFullscreen) await root.requestFullscreen();
      else if (root.webkitRequestFullscreen) root.webkitRequestFullscreen();
    }
  } catch {}
  syncFullscreen();
}

function fullscreenLabel() {
  const lang = state.value?.settings?.language || "en";
  const active = fullscreen.value;
  if (lang === "pt") return active ? "Sair" : "Tela cheia";
  return active ? "Exit" : "Fullscreen";
}

function onHudKeyDown(e) {
  if (e.key !== "Escape" || !openPanel.value) return;
  e.preventDefault();
  e.stopPropagation();
  if (detailSkill.value) {
    closeSkillDetails();
    return;
  }
  closePanel();
}

onMounted(() => {
  syncFullscreen();
  document.addEventListener("fullscreenchange", syncFullscreen);
  document.addEventListener("webkitfullscreenchange", syncFullscreen);
  window.addEventListener("keydown", onHudKeyDown, true);
});
onBeforeUnmount(() => {
  document.removeEventListener("fullscreenchange", syncFullscreen);
  document.removeEventListener("webkitfullscreenchange", syncFullscreen);
  window.removeEventListener("keydown", onHudKeyDown, true);
});

function pct(current, max) {
  if (!max || max <= 0) return 0;
  return Math.max(0, Math.min(100, (current / max) * 100));
}
function act(code, a = 0, b = 0, c = 0) { uiAction(code, a, b, c); }
function closePanel() {
  closeMenuAndFocusGame(() => {
    openSkillCalc.value = null;
    detailSkill.value = null;
    openPanel.value = null;
  });
}
function togglePanel(panel) {
  if (openPanel.value === panel) {
    closePanel();
  } else {
    openSkillCalc.value = null;
    detailSkill.value = null;
    openPanel.value = panel;
  }
}
function closePanelOnOverlay(e) {
  if (e.target === e.currentTarget) closePanel();
}

const needsDistribution = computed(() => !!(player.value?.pending_level_up || (player.value?.points || 0) > 0));
const tabs = computed(() => [
  ["status", labels.value.status || "Status", needsDistribution.value],
  ["equipment", labels.value.equipment || "Equipment", false],
  ["skills", labels.value.skills || "Skills", false],
  ["items", labels.value.inventory || "Items", false],
]);

const SKIN_COLORS = ["#f6d7b0", "#efc89a", "#ddb07a", "#c89262", "#a8744c", "#86583a", "#f0e0c8", "#f2b8a8"];
// Keep in sync with runtime tints in crates/enzo-wasm/src/rig.rs
const HAIR_COLORS = ["#5a3a22", "#1f1a1a", "#5c3d29", "#d1ad4d", "#9e3830", "#c7ccd9", "#4d85e0", "#b861d1"];
const CLOTH_COLORS = ["#4f7cff", "#4f84ff", "#f274a8", "#4dc773", "#f2b847", "#e6594d", "#8c73d9", "#c7ccd9"];

const CUSTOM_SECTIONS = [
  { titleKey: "character", fields: [] },
  { titleKey: "skin_tone", fields: [{ code: 0, key: "skin_tone", max: 7, kind: "color", palette: SKIN_COLORS }] },
  { titleKey: "hair", fields: [
    { code: 1, key: "hair_style", labelKey: "hair_style", max: 7, kind: "style" },
    { code: 2, key: "hair_color", labelKey: "hair_color", max: 7, kind: "color", palette: HAIR_COLORS },
  ]},
  { titleKey: "face", fields: [{ code: 3, key: "face", max: 7, kind: "style" }] },
  { titleKey: "clothing_colors", fields: [
    { code: 4, key: "outfit_top", labelKey: "outfit_top", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 7, key: "outfit_top_detail1", labelKey: "outfit_top_detail1", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 8, key: "outfit_top_detail2", labelKey: "outfit_top_detail2", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 5, key: "outfit_bottom", labelKey: "outfit_bottom", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 6, key: "shoes", labelKey: "shoes", max: 7, kind: "color", palette: CLOTH_COLORS },
  ]},
];

function profileValue(key) {
  return player.value?.visual_profile?.[key] ?? 0;
}
function setProfile(code, value) { setVisualProfileField(code, value); }
function cycleProfile(field, dir) {
  const max = (field.max ?? 7) + 1;
  const next = (profileValue(field.key) + dir + max) % max;
  setVisualProfileField(field.code, next);
}
function fieldLabel(field, fallbackKey) {
  const k = field.labelKey || fallbackKey || field.key;
  const isEnglish = (state.value?.settings?.language || "pt-BR") === "en";
  const fallbacks = {
    outfit_top: isEnglish ? "Shirt color (predominant)" : "Cor da camiseta (predominante)",
    outfit_bottom: isEnglish ? "Pants color (predominant)" : "Cor da calça (predominante)",
    shoes: isEnglish ? "Details color" : "Cor dos detalhes",
  };
  return labels.value[k] || fallbacks[k] || k.replace(/_/g, " ");
}
function sectionTitle(section) {
  const isEnglish = (state.value?.settings?.language || "pt-BR") === "en";
  const map = {
    hair: labels.value.hair_style || (isEnglish ? "Hair" : "Cabelo"),
    clothing_colors: isEnglish ? "Clothing colors (top, pants, details)" : "Cores das roupas (cima, calça, detalhes)",
  };
  return labels.value[section.titleKey] || map[section.titleKey] || section.titleKey.replace(/_/g, " ");
}
function swatchColor(field) {
  if (!field.palette) return null;
  return field.palette[profileValue(field.key) % field.palette.length];
}

const attrDefs = computed(() => {
  const attrs = player.value?.attributes || {};
  return [
    ["Speed", attrs.speed ?? 0, 0],
    ["Strength", attrs.strength ?? 0, 1],
    ["Health", attrs.health ?? 0, 2],
    ["Aura", attrs.aura ?? 0, 3],
    ["Ego", attrs.ego ?? 0, 4],
  ];
});

const catalogSkills = computed(() => player.value?.catalog || []);

function openSkillDetails(entry) {
  openSkillCalc.value = null;
  detailSkill.value = entry;
}
function closeSkillDetails() {
  openSkillCalc.value = null;
  detailSkill.value = null;
}
function toggleSkillCalc(key) {
  openSkillCalc.value = openSkillCalc.value === key ? null : key;
}
function formatDamagePreview(entry) {
  const values = entry?.damage_preview || [];
  if (!values.length) return labels.value.skill_no_damage || "—";
  const fmt = (v) => (Number.isInteger(v) ? String(v) : Number(v).toFixed(1).replace(/\.0$/, ""));
  return values.map(fmt).join(" / ");
}
function formatCooldown(sec) {
  if (sec == null) return "—";
  const n = Number(sec);
  if (!Number.isFinite(n) || n <= 0) return "0s";
  return `${n.toFixed(2).replace(/\.00$/, "").replace(/(\.\d)0$/, "$1")}s`;
}
function formatAuraCost(cost) {
  if (cost == null) return "0";
  const n = Number(cost);
  if (!Number.isFinite(n)) return "0";
  return Number.isInteger(n) ? String(n) : n.toFixed(1).replace(/\.0$/, "");
}
const showActionBar = computed(() => showHud.value && player.value && !mobileControlsVisible(state.value?.settings, state.value?.scene));
const showPhaseOverlay = computed(() => showHud.value && (phase.value === "Dead" || phase.value === "Victory"));

const controls = computed(() => state.value?.controls || []);
function primaryKeyForAction(actionName) {
  const entry = controls.value.find((c) => c && c.action === actionName);
  if (entry && Array.isArray(entry.keys) && entry.keys.length > 0) return entry.keys[0];
  return "";
}
function slotKeyLabel(index) {
  return primaryKeyForAction(`Slot${index + 1}`) || String(index + 1);
}

function skillMeta(entry) {
  const parts = [entry.category || ""];
  if (entry.equipped_slot != null) parts.push(`${labels.value.equipped || "Slot"} ${entry.equipped_slot + 1}`);
  if (entry.aura_cost > 0) parts.push(`Aura ${entry.aura_cost}`);
  if (entry.cooldown_max > 0) parts.push(`CD ${entry.cooldown_max}s`);
  return parts.filter(Boolean).join(" · ");
}
function slotSkillName(slotState) {
  return slotState?.skill_name || (slotState?.skill != null ? `S${slotState.skill}` : "—");
}

function uiLang() {
  const lang = state.value?.settings?.language || "pt-BR";
  return String(lang).toLowerCase().startsWith("en") ? "en" : "pt";
}

function l(key, en, pt) {
  if (labels.value && labels.value[key]) return labels.value[key];
  return uiLang() === "pt" ? pt : en;
}

function calcUiLabel() { return l("skill_calc", "Calculation", "Cálculo"); }
function rangeUiLabel() { return l("range", "Range", "Alcance"); }
function knockbackUiLabel() { return l("knockback", "Knockback", "Repulsão"); }
function seeCalcUiLabel() { return l("see_calc", "See calc", "Ver cálculo"); }

function translateSkillCalc(text, lang = uiLang()) {
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
</script>

<template>
  <div id="dom-hud-root" class="dom-hud-root" :style="{ pointerEvents: 'none' }">
    <template v-if="showHud && player">
      <div class="dom-hud-top">
        <div class="dom-hud-bars">
          <div class="dom-hud-bar-row">
            <span class="dom-hud-bar-label">HP {{ Math.ceil(player.health) }}/{{ Math.ceil(player.max_health) }}</span>
            <div class="dom-hud-track hp"><div class="dom-hud-fill" :style="{ width: pct(player.health, player.max_health) + '%' }" /></div>
          </div>
          <div class="dom-hud-bar-row">
            <span class="dom-hud-bar-label">Aura {{ Math.ceil(player.aura) }}/{{ Math.ceil(player.max_aura) }}</span>
            <div class="dom-hud-track aura"><div class="dom-hud-fill" :style="{ width: pct(player.aura, player.max_aura) + '%' }" /></div>
          </div>
          <div class="dom-hud-bar-row">
            <span class="dom-hud-bar-label">Lv {{ player.level }} {{ Math.floor(player.xp) }}/{{ Math.floor(player.xp_to_next) }}</span>
            <div class="dom-hud-track xp"><div class="dom-hud-fill" :style="{ width: pct(player.xp, player.xp_to_next) + '%' }" /></div>
          </div>
        </div>

        <div class="dom-hud-tabs">
          <button
            v-for="[id, label, blink] in tabs"
            :key="id"
            type="button"
            class="dom-hud-tab"
            :class="{ active: openPanel === id, blink }"
            @click.prevent.stop="togglePanel(id)"
          >{{ label }}</button>
        </div>

        <div class="dom-hud-actions" style="pointer-events: auto">
          <button
            type="button"
            class="dom-hud-btn dom-hud-fullscreen-btn"
            :data-lang="state.settings?.language || 'en'"
            :title="fullscreenLabel()"
            :aria-pressed="fullscreen ? 'true' : 'false'"
            @click.prevent.stop="void toggleFullscreen()"
          >{{ fullscreenLabel() }}</button>
          <button
            type="button"
            class="dom-hud-btn dom-hud-menu-btn"
            @click.prevent.stop="act(UiAction.HudOpenMenu)"
          >{{ labels.menu || 'Menu' }}</button>
        </div>
      </div>

      <div v-if="showActionBar" class="dom-action-bar" style="pointer-events: auto">
        <button
          v-for="(slot, index) in (player.slots || [])"
          :key="index"
          type="button"
          class="dom-action-slot"
          :class="{ active: slot.active, locked: !slot.available, cooldown: slot.cooldown > 0.01 }"
          :title="slot.lock_reason || ''"
          @click.prevent.stop="togglePanel('skills')"
        >
          <span class="slot-num">{{ slotKeyLabel(index) }}</span>
          <img v-if="slot.skill_icon" class="slot-skill-icon" :src="slot.skill_icon" :alt="slot.skill_name || ''" :title="slot.skill_name || ''" />
          <span v-else class="slot-skill">—</span>
          <span v-if="slot.cooldown > 0.01" class="slot-cd">{{ slot.cooldown.toFixed(1) }}</span>
        </button>
      </div>

      <div v-if="openPanel" class="dom-game-panel-overlay" @click="closePanelOnOverlay">
        <div class="dom-game-panel" style="pointer-events: auto" @click.stop>
          <template v-if="openPanel === 'status'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.status_title || labels.status || 'Status' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="closePanel()">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <div class="dom-status-grid">
                <div class="dom-status-row"><span class="dom-status-key">{{ labels.level || 'Level' }}</span><span class="dom-status-val">{{ player.level }}</span></div>
                <div class="dom-status-row"><span class="dom-status-key">XP</span><span class="dom-status-val">{{ Math.floor(player.xp) }} / {{ Math.floor(player.xp_to_next) }}</span></div>
                <div class="dom-status-row"><span class="dom-status-key">{{ labels.points || 'Points' }}</span><span class="dom-status-val">{{ player.points }}</span></div>
                <div class="dom-status-row"><span class="dom-status-key">HP</span><span class="dom-status-val">{{ Math.ceil(player.health) }} / {{ Math.ceil(player.max_health) }}</span></div>
                <div class="dom-status-row"><span class="dom-status-key">Aura</span><span class="dom-status-val">{{ Math.ceil(player.aura) }} / {{ Math.ceil(player.max_aura) }}</span></div>
              </div>

              <h4 class="dom-subtitle">{{ labels.points || 'Points' }}</h4>
              <template v-if="(player.points || 0) > 0">
                <p class="muted">{{ labels.points || 'Points' }}: {{ player.points }}</p>
                <div class="dom-attr-grid">
                  <button
                    v-for="[name, value, code] in attrDefs"
                    :key="code"
                    type="button"
                    class="dom-btn small"
                    style="pointer-events:auto; touch-action: manipulation"
                    @click.prevent.stop="act(UiAction.HudAllocate, code)"
                  >+ {{ name }} ({{ value }})</button>
                </div>
              </template>
              <div v-else class="dom-status-grid">
                <div v-for="[name, value] in attrDefs" :key="name" class="dom-status-row">
                  <span class="dom-status-key">{{ name }}</span><span class="dom-status-val">{{ value }}</span>
                </div>
              </div>
            </div>
          </template>

          <template v-else-if="openPanel === 'equipment'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.equipment_title || labels.equipment || 'Equipment' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="closePanel()">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <p v-if="!(player.equipment_slots || []).length" class="muted">{{ labels.no_item || '' }}</p>
              <div v-else class="dom-list">
                <div v-for="slot in (player.equipment_slots || [])" :key="slot.slot" class="dom-list-row">
                  <span class="dom-list-key">{{ slot.slot_label || slot.slot }}</span>
                  <span class="dom-list-val">{{ slot.item || labels.no_item || '—' }}</span>
                  <button
                    v-if="slot.item_code != null"
                    type="button"
                    class="dom-btn small secondary"
                    style="margin-left:auto; pointer-events:auto; touch-action: manipulation"
                    @click.prevent.stop="unequipSlot(slot.slot_code)"
                  >{{ labels.unequip || 'Unequip' }}</button>
                </div>
              </div>
            </div>
          </template>

          <template v-else-if="openPanel === 'items'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.inventory_title || labels.inventory || 'Items' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="closePanel()">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <p v-if="!(player.inventory || []).length" class="muted">{{ labels.inventory_empty || 'Empty' }}</p>
              <div v-else class="dom-list">
                <div v-for="(item, i) in (player.inventory || [])" :key="i" class="dom-list-row">
                  <span class="dom-list-val">{{ item.name || item.kind }}</span>
                  <button
                    v-if="item.code != null"
                    type="button"
                    class="dom-btn small primary"
                    style="margin-left:auto; pointer-events:auto; touch-action: manipulation"
                    @click.prevent.stop="equipItem(item.code)"
                  >{{ labels.equip || 'Equip' }}</button>
                </div>
              </div>
            </div>
          </template>

          <template v-else-if="openPanel === 'customize'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.customize || 'Customize Character' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="closePanel()">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <p class="muted" style="margin:0 0 8px 0">
                {{ (state.settings?.language || 'pt-BR') === 'en'
                  ? 'Clothing uses 3 colors: shirt (predominant), pants (predominant), and details.'
                  : 'As roupas usam 3 cores: cor da camiseta (predominante), calça (predominante) e detalhes.' }}
              </p>
              <div class="dom-list">
                <div class="dom-list-row" style="align-items:center; gap:10px; flex-wrap:wrap;">
                  <span class="dom-list-key">{{ labels.character || 'Character' }}</span>
                  <div style="display:flex; gap:8px; pointer-events:auto">
                    <button type="button" class="dom-btn small" :class="{ primary: player.character_code === 0 }" @click.prevent.stop="setPlayableCharacter(0)">Enzo</button>
                    <button type="button" class="dom-btn small" :class="{ primary: player.character_code === 1 }" @click.prevent.stop="setPlayableCharacter(1)">Valentina</button>
                  </div>
                  <span class="dom-list-val" style="margin-left:auto">{{ player.character || 'Enzo' }}</span>
                </div>

                <template v-for="section in CUSTOM_SECTIONS" :key="section.titleKey">
                  <template v-if="section.fields.length">
                    <div class="dom-list-row" style="margin-top:8px; opacity:0.9; font-weight:600; font-size:12px; text-transform:uppercase; letter-spacing:.04em">{{ sectionTitle(section) }}</div>
                    <div v-for="field in section.fields" :key="field.code" class="dom-list-row" style="align-items:center; gap:8px;">
                      <span class="dom-list-key">{{ fieldLabel(field, section.titleKey) }}</span>
                      <span v-if="swatchColor(field)" :style="{ width:'18px', height:'18px', borderRadius:'50%', background: swatchColor(field), border:'2px solid rgba(255,255,255,.25)', flex:'0 0 auto' }"></span>
                      <span class="dom-list-val">{{ profileValue(field.key) }}</span>
                      <div style="margin-left:auto; display:flex; gap:6px; pointer-events:auto">
                        <button type="button" class="dom-btn small secondary" @click.prevent.stop="cycleProfile(field, -1)">−</button>
                        <button type="button" class="dom-btn small" @click.prevent.stop="cycleProfile(field, 1)">+</button>
                      </div>
                    </div>
                    <div v-if="section.fields.some(f => f.palette)" style="display:flex; gap:6px; flex-wrap:wrap; padding:4px 0 2px 0; pointer-events:auto">
                      <template v-for="field in section.fields.filter(f => f.palette)" :key="'p'+field.code">
                        <button v-for="(c, i) in field.palette" :key="field.code+':'+i" type="button"
                          @click.prevent.stop="setProfile(field.code, i)"
                          :title="fieldLabel(field, section.titleKey) + ' ' + i"
                          :style="{ width:'22px', height:'22px', borderRadius:'6px', background:c, border: profileValue(field.key)===i ? '2px solid #fff' : '2px solid rgba(255,255,255,.15)', cursor:'pointer', padding:0 }"></button>
                      </template>
                    </div>
                  </template>
                </template>
              </div>
              <p class="muted" style="margin-top:10px">{{ labels.customize_hint || 'Choose character (Enzo/Valentina), hair, face, clothing, and skin tone. Equipped items render as overlays.' }}</p>
            </div>
          </template>

          <template v-else-if="openPanel === 'skills'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.skills_title || labels.skills || 'Skills' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="closePanel()">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <p class="dom-subtitle">{{ labels.assign || 'Assign to slot' }}</p>
              <div class="dom-slot-picker">
                <button
                  v-for="i in [0,1,2,3]"
                  :key="i"
                  type="button"
                  class="dom-btn small dom-slot-pick-btn"
                  :class="{ selected: selectedSkillSlot === i }"
                  @click.prevent.stop="selectedSkillSlot = i"
                >
                  <span>{{ slotKeyLabel(i) }}:</span>
                  <template v-if="(player.slots || []).find(s => s.slot === i)?.skill_icon">
                    <img
                      class="dom-skill-icon sm"
                      :src="(player.slots || []).find(s => s.slot === i).skill_icon"
                      :alt="(player.slots || []).find(s => s.slot === i)?.skill_name || ''"
                      :title="(player.slots || []).find(s => s.slot === i)?.skill_name || ''"
                    />
                    <span>{{ (player.slots || []).find(s => s.slot === i)?.skill_name || '—' }}</span>
                  </template>
                  <span v-else>—</span>
                </button>
              </div>
              <div class="dom-option-row">
                <button type="button" class="dom-btn small danger" @click.prevent.stop="loadoutClear(selectedSkillSlot)">{{ labels.clear || 'Clear' }}</button>
              </div>
              <p v-if="!catalogSkills.length" class="muted">{{ labels.locked || 'Locked' }}</p>
              <div v-else class="dom-skill-list">
                <div
                  v-for="entry in catalogSkills"
                  :key="entry.skill"
                  class="dom-skill-row"
                  :class="{ equipped: entry.equipped_slot != null, locked: !entry.available }"
                >
                  <div class="dom-skill-info dom-skill-info--icon">
                    <img v-if="entry.icon" class="dom-skill-icon" :src="entry.icon" :alt="entry.name || ''" :title="entry.name || ''" />
                    <div>
                      <div class="dom-skill-name">{{ entry.name || `Skill ${entry.skill}` }}</div>
                      <div class="dom-skill-meta">{{ skillMeta(entry) }}</div>
                    </div>
                  </div>
                  <div class="dom-skill-actions" style="display:flex; gap:6px; flex-wrap:wrap; justify-content:flex-end">
                    <button type="button" class="dom-btn small secondary" @click.prevent.stop="openSkillDetails(entry)">{{ labels.skill_details || 'Details' }}</button>
                    <button
                      type="button"
                      class="dom-btn small primary"
                      :disabled="!entry.available"
                      @click.prevent.stop="entry.available && loadoutAssign(selectedSkillSlot, entry.skill)"
                    >{{ labels.assign || 'Assign' }}</button>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="detailSkill" class="dom-modal-overlay dom-skill-detail-overlay" @click.self.prevent.stop="closeSkillDetails()">
              <div class="dom-modal dom-skill-detail-modal" @click.stop>
                <div class="dom-help-header">
                  <div style="display:flex; align-items:center; gap:10px; min-width:0">
                    <img v-if="detailSkill.icon" class="dom-skill-icon" :src="detailSkill.icon" :alt="detailSkill.name || ''" />
                    <h3 class="dom-title" style="margin:0">{{ detailSkill.name }}</h3>
                  </div>
                  <button type="button" class="dom-btn small secondary" @click.prevent.stop="closeSkillDetails()">{{ labels.close || 'Close' }}</button>
                </div>
                <div class="dom-help-body">
                  <p class="muted" style="margin:0 0 10px 0; white-space:pre-wrap">{{ detailSkill.description || '' }}</p>
                  <div class="dom-status-grid">
                    <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_level || 'Level' }}</span><span class="dom-status-val">{{ detailSkill.effective_level ?? 0 }}</span></div>
                    <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_tier || 'Active tier' }}</span><span class="dom-status-val">{{ detailSkill.active_tier != null ? detailSkill.active_tier : '—' }}</span></div>

                    <div class="dom-status-row dom-status-row--calc">
                      <span class="dom-status-key">{{ labels.skill_damage || 'Damage' }}</span>
                      <span class="dom-status-val dom-status-val--with-calc">
                        <span>{{ formatDamagePreview(detailSkill) }}</span>
                        <button
                          v-if="detailSkill.damage_calc"
                          type="button"
                          class="dom-calc-btn"
                          :class="{ active: openSkillCalc === 'damage' }"
                          :title="calcUiLabel()"
                          @click.prevent.stop="toggleSkillCalc('damage')"
                        >!</button>
                      </span>
                    </div>
                    <div v-if="openSkillCalc === 'damage' && detailSkill.damage_calc" class="dom-calc-box">
                      <div class="dom-calc-title">{{ calcUiLabel() }} · {{ labels.skill_damage || (uiLang() === 'pt' ? 'Dano' : 'Damage') }}</div>
                      <pre class="dom-calc-text">{{ translateSkillCalc(detailSkill.damage_calc) }}</pre>
                    </div>

                    <div v-if="detailSkill.range_calc" class="dom-status-row dom-status-row--calc">
                      <span class="dom-status-key">{{ rangeUiLabel() }}</span>
                      <span class="dom-status-val dom-status-val--with-calc">
                        <span>{{ seeCalcUiLabel() }}</span>
                        <button
                          type="button"
                          class="dom-calc-btn"
                          :class="{ active: openSkillCalc === 'range' }"
                          :title="calcUiLabel()"
                          @click.prevent.stop="toggleSkillCalc('range')"
                        >!</button>
                      </span>
                    </div>
                    <div v-if="detailSkill.range_calc && openSkillCalc === 'range'" class="dom-calc-box">
                      <div class="dom-calc-title">{{ calcUiLabel() }} · {{ rangeUiLabel() }}</div>
                      <pre class="dom-calc-text">{{ translateSkillCalc(detailSkill.range_calc) }}</pre>
                    </div>

                    <div v-if="detailSkill.knockback_calc" class="dom-status-row dom-status-row--calc">
                      <span class="dom-status-key">{{ knockbackUiLabel() }}</span>
                      <span class="dom-status-val dom-status-val--with-calc">
                        <span>{{ seeCalcUiLabel() }}</span>
                        <button
                          type="button"
                          class="dom-calc-btn"
                          :class="{ active: openSkillCalc === 'knockback' }"
                          :title="calcUiLabel()"
                          @click.prevent.stop="toggleSkillCalc('knockback')"
                        >!</button>
                      </span>
                    </div>
                    <div v-if="detailSkill.knockback_calc && openSkillCalc === 'knockback'" class="dom-calc-box">
                      <div class="dom-calc-title">{{ calcUiLabel() }} · {{ knockbackUiLabel() }}</div>
                      <pre class="dom-calc-text">{{ translateSkillCalc(detailSkill.knockback_calc) }}</pre>
                    </div>

                    <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_aura_cost || 'Aura cost' }}</span><span class="dom-status-val">{{ formatAuraCost(detailSkill.aura_cost) }}</span></div>
                    <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_cooldown || 'Cooldown' }}</span><span class="dom-status-val">{{ formatCooldown(detailSkill.cooldown_max) }}</span></div>
                    <div v-if="detailSkill.category" class="dom-status-row"><span class="dom-status-key">Type</span><span class="dom-status-val">{{ detailSkill.category }}</span></div>
                    <div v-if="detailSkill.equipped_slot != null" class="dom-status-row"><span class="dom-status-key">{{ labels.equipped || 'Equipped' }}</span><span class="dom-status-val">{{ labels.equipped || 'Slot' }} {{ detailSkill.equipped_slot + 1 }}</span></div>
                    <div v-if="!detailSkill.available && detailSkill.lock_reason" class="dom-status-row"><span class="dom-status-key">{{ labels.locked || 'Locked' }}</span><span class="dom-status-val">{{ detailSkill.lock_reason }}</span></div>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>
    </template>

    <div v-if="showHud && state.game_toast" class="dom-game-toast">{{ state.game_toast }}</div>

    <div v-if="showPhaseOverlay" class="dom-phase-overlay">
      <div class="dom-phase-panel">
        <h2 class="dom-title">{{ phase === 'Dead' ? (labels.game_over || 'Game Over') : (labels.victory || 'Victory') }}</h2>
        <div class="dom-modal-actions">
          <button type="button" class="dom-btn primary" @click.prevent.stop="act(UiAction.HudRestart)">{{ labels.restart || 'Restart' }}</button>
          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.HudOpenMenu)">{{ labels.menu || 'Menu' }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
