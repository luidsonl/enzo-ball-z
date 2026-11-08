<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { UiAction, equipItem, loadoutAssign, loadoutClear, setPlayableCharacter, setVisualProfileField, uiAction, unequipSlot } from "../../bridge/bridge.js";
import { mobileControlsVisible } from "../../lib/mobile.js";

const props = defineProps({
  state: { type: Object, default: null },
});

const openPanel = ref(null); // status | equipment | skills | items | null
const selectedSkillSlot = ref(0);
const fullscreen = ref(false);

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

onMounted(() => {
  syncFullscreen();
  document.addEventListener("fullscreenchange", syncFullscreen);
  document.addEventListener("webkitfullscreenchange", syncFullscreen);
});
onBeforeUnmount(() => {
  document.removeEventListener("fullscreenchange", syncFullscreen);
  document.removeEventListener("webkitfullscreenchange", syncFullscreen);
});

function pct(current, max) {
  if (!max || max <= 0) return 0;
  return Math.max(0, Math.min(100, (current / max) * 100));
}
function act(code, a = 0, b = 0, c = 0) { uiAction(code, a, b, c); }
function togglePanel(panel) {
  openPanel.value = openPanel.value === panel ? null : panel;
}
function closePanelOnOverlay(e) {
  if (e.target === e.currentTarget) openPanel.value = null;
}

const needsDistribution = computed(() => !!(player.value?.pending_level_up || (player.value?.points || 0) > 0));
const tabs = computed(() => [
  ["status", labels.value.status || "Status", needsDistribution.value],
  ["equipment", labels.value.equipment || "Equipment", false],
  ["skills", labels.value.skills || "Skills", false],
  ["items", labels.value.inventory || "Items", false],
  ["customize", labels.value.customize || "Customize", false],
]);

const PROFILE_FIELDS = [
  { code: 0, key: "skin_tone", labelKey: "skin_tone", max: 7 },
  { code: 1, key: "hair_style", labelKey: "hair_style", max: 7 },
  { code: 2, key: "hair_color", labelKey: "hair_color", max: 7 },
  { code: 3, key: "face", labelKey: "face", max: 7 },
  { code: 4, key: "outfit_top", labelKey: "outfit_top", max: 7 },
  { code: 5, key: "outfit_bottom", labelKey: "outfit_bottom", max: 7 },
  { code: 6, key: "shoes", labelKey: "shoes", max: 7 },
];

function profileValue(field) {
  return player.value?.visual_profile?.[field.key] ?? 0;
}
function cycleProfile(field, dir) {
  const max = field.max + 1;
  const next = (profileValue(field) + dir + max) % max;
  setVisualProfileField(field.code, next);
}
function profileLabel(field) {
  return labels.value[field.labelKey] || field.key;
}

const attrDefs = computed(() => {
  const attrs = player.value?.attributes || {};
  return [
    ["Speed", attrs.speed ?? 0, 0],
    ["Strength", attrs.strength ?? 0, 1],
    ["Health", attrs.health ?? 0, 2],
    ["Aura", attrs.aura ?? 0, 3],
  ];
});

const unlockedSkills = computed(() => (player.value?.catalog || []).filter((entry) => entry.available));
const showActionBar = computed(() => showHud.value && player.value && !mobileControlsVisible(state.value?.settings, state.value?.scene));
const showPhaseOverlay = computed(() => showHud.value && (phase.value === "Dead" || phase.value === "Victory"));

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
          <span class="slot-num">{{ index + 1 }}</span>
          <span class="slot-skill">{{ slot.skill_name || (slot.skill != null ? `S${slot.skill}` : '—') }}</span>
          <span v-if="slot.cooldown > 0.01" class="slot-cd">{{ slot.cooldown.toFixed(1) }}</span>
        </button>
      </div>

      <div v-if="openPanel" class="dom-game-panel-overlay" @click="closePanelOnOverlay">
        <div class="dom-game-panel" style="pointer-events: auto" @click.stop>
          <template v-if="openPanel === 'status'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.status_title || labels.status || 'Status' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="openPanel = null">{{ labels.close || 'Close' }}</button>
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
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="openPanel = null">{{ labels.close || 'Close' }}</button>
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
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="openPanel = null">{{ labels.close || 'Close' }}</button>
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
              <h3 class="dom-title">{{ labels.customize || 'Customize' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="openPanel = null">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <div class="dom-list">
                <div class="dom-list-row">
                  <span class="dom-list-key">{{ labels.character || 'Character' }}</span>
                  <span class="dom-list-val">{{ player.character || 'Enzo' }}</span>
                  <div style="margin-left:auto; display:flex; gap:6px; pointer-events:auto">
                    <button type="button" class="dom-btn small" :class="{ primary: player.character_code === 0 }" @click.prevent.stop="setPlayableCharacter(0)">Enzo</button>
                    <button type="button" class="dom-btn small" :class="{ primary: player.character_code === 1 }" @click.prevent.stop="setPlayableCharacter(1)">Valentina</button>
                  </div>
                </div>
                <div v-for="field in PROFILE_FIELDS" :key="field.code" class="dom-list-row">
                  <span class="dom-list-key">{{ profileLabel(field) }}</span>
                  <span class="dom-list-val">{{ profileValue(field) }}</span>
                  <div style="margin-left:auto; display:flex; gap:6px; pointer-events:auto">
                    <button type="button" class="dom-btn small secondary" @click.prevent.stop="cycleProfile(field, -1)">-</button>
                    <button type="button" class="dom-btn small" @click.prevent.stop="cycleProfile(field, 1)">+</button>
                  </div>
                </div>
              </div>
              <p class="muted" style="margin-top:10px">Layers: skin/face/hair, shirt, pants, shoes, plus equipped items (weapon, cap, armor, sneakers).</p>
            </div>
          </template>

          <template v-else-if="openPanel === 'skills'">
            <div class="dom-game-panel-header">
              <h3 class="dom-title">{{ labels.skills_title || labels.skills || 'Skills' }}</h3>
              <button type="button" class="dom-btn small secondary" @click.prevent.stop="openPanel = null">{{ labels.close || 'Close' }}</button>
            </div>
            <div class="dom-game-panel-body">
              <p class="dom-subtitle">{{ labels.assign || 'Assign to slot' }}</p>
              <div class="dom-slot-picker">
                <button
                  v-for="i in [0,1,2,3]"
                  :key="i"
                  type="button"
                  class="dom-btn small"
                  :class="{ selected: selectedSkillSlot === i }"
                  @click.prevent.stop="selectedSkillSlot = i"
                >{{ i + 1 }}: {{ slotSkillName((player.slots || []).find(s => s.slot === i)) }}</button>
              </div>
              <div class="dom-option-row">
                <button type="button" class="dom-btn small danger" @click.prevent.stop="loadoutClear(selectedSkillSlot)">{{ labels.clear || 'Clear' }}</button>
              </div>
              <p v-if="!unlockedSkills.length" class="muted">{{ labels.locked || 'Locked' }}</p>
              <div v-else class="dom-skill-list">
                <div
                  v-for="entry in unlockedSkills"
                  :key="entry.skill"
                  class="dom-skill-row"
                  :class="{ equipped: entry.equipped_slot != null }"
                >
                  <div class="dom-skill-info">
                    <div class="dom-skill-name">{{ entry.name || `Skill ${entry.skill}` }}</div>
                    <div class="dom-skill-meta">{{ skillMeta(entry) }}</div>
                  </div>
                  <div class="dom-skill-actions">
                    <button type="button" class="dom-btn small primary" @click.prevent.stop="loadoutAssign(selectedSkillSlot, entry.skill)">{{ labels.assign || 'Assign' }}</button>
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
