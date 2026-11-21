<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import PanelHeader from "../../../molecules/PanelHeader.vue";
import AppButton from "../../../atoms/AppButton.vue";
import AppTitle from "../../../atoms/AppTitle.vue";
import {
  calcUiLabel,
  formatAuraCost,
  formatCooldown,
  formatDamagePreview,
  knockbackUiLabel,
  rangeUiLabel,
  seeCalcUiLabel,
  translateSkillCalc,
  uiLangFromState,
} from "../../../../lib/skillCalc.js";

const props = defineProps({
  state: { type: Object, default: null },
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
  selectedSkillSlot: { type: Number, default: 0 },
  slotKeyLabel: { type: Function, required: true },
});
const emit = defineEmits(["close", "select-slot", "assign", "clear"]);

const detailSkill = ref(null);
const openSkillCalc = ref(null);

const visibleCatalogSkills = computed(() =>
  (props.player?.catalog || []).filter((entry) => entry && entry.available)
);

function skillMeta(entry) {
  const parts = [entry.category || ""];
  if (entry.equipped_slot != null) parts.push(`${props.labels.equipped || "Slot"} ${entry.equipped_slot + 1}`);
  if (entry.aura_cost > 0) parts.push(`Aura ${entry.aura_cost}`);
  if (entry.cooldown_max > 0) parts.push(`CD ${entry.cooldown_max}s`);
  return parts.filter(Boolean).join(" · ");
}

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

function trCalc(text) {
  return translateSkillCalc(text, uiLangFromState(props.state));
}

function onKeyDown(e) {
  if (e.key === "Escape" && detailSkill.value) {
    e.preventDefault();
    e.stopPropagation();
    if (typeof e.stopImmediatePropagation === "function") e.stopImmediatePropagation();
    closeSkillDetails();
  }
}

onMounted(() => window.addEventListener("keydown", onKeyDown, true));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeyDown, true));

defineExpose({ closeSkillDetails, hasDetailOpen: () => !!detailSkill.value });
</script>

<template>
  <PanelHeader
    :title="labels.skills_title || labels.skills || 'Skills'"
    :close-label="labels.close || 'Close'"
    @close="$emit('close')"
  />
  <div class="dom-game-panel-body">
    <p class="dom-subtitle">{{ labels.assign || 'Assign to slot' }}</p>
    <div class="dom-slot-picker">
      <button
        v-for="i in [0,1,2,3]"
        :key="i"
        type="button"
        class="dom-btn small dom-slot-pick-btn"
        :class="{ selected: selectedSkillSlot === i }"
        @click.prevent.stop="$emit('select-slot', i)"
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
      <AppButton size="small" variant="danger" @click.prevent.stop="$emit('clear', selectedSkillSlot)">
        {{ labels.clear || 'Clear' }}
      </AppButton>
    </div>

    <p v-if="!visibleCatalogSkills.length" class="muted">{{ labels.locked || 'Locked' }}</p>
    <div v-else class="dom-skill-list">
      <div
        v-for="entry in visibleCatalogSkills"
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
          <AppButton size="small" variant="secondary" @click.prevent.stop="openSkillDetails(entry)">
            {{ labels.skill_details || 'Details' }}
          </AppButton>
          <AppButton
            size="small"
            variant="primary"
            :disabled="!entry.available"
            @click.prevent.stop="entry.available && $emit('assign', selectedSkillSlot, entry.skill)"
          >{{ labels.assign || 'Assign' }}</AppButton>
        </div>
      </div>
    </div>
  </div>

  <div v-if="detailSkill" class="dom-modal-overlay dom-skill-detail-overlay" @click.self.prevent.stop="closeSkillDetails()">
    <div class="dom-modal dom-skill-detail-modal" @click.stop>
      <div class="dom-help-header">
        <div style="display:flex; align-items:center; gap:10px; min-width:0">
          <img v-if="detailSkill.icon" class="dom-skill-icon" :src="detailSkill.icon" :alt="detailSkill.name || ''" />
          <AppTitle tag="h3" style="margin:0">{{ detailSkill.name }}</AppTitle>
        </div>
        <AppButton size="small" variant="secondary" @click.prevent.stop="closeSkillDetails()">{{ labels.close || 'Close' }}</AppButton>
      </div>
      <div class="dom-help-body">
        <p class="muted" style="margin:0 0 10px 0; white-space:pre-wrap">{{ detailSkill.description || '' }}</p>
        <div class="dom-status-grid">
          <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_level || 'Level' }}</span><span class="dom-status-val">{{ detailSkill.effective_level ?? 0 }}</span></div>
          <div class="dom-status-row"><span class="dom-status-key">{{ labels.skill_tier || 'Active tier' }}</span><span class="dom-status-val">{{ detailSkill.active_tier != null ? detailSkill.active_tier : '—' }}</span></div>

          <div class="dom-status-row dom-status-row--calc">
            <span class="dom-status-key">{{ labels.skill_damage || 'Damage' }}</span>
            <span class="dom-status-val dom-status-val--with-calc">
              <span>{{ formatDamagePreview(detailSkill, state) }}</span>
              <button
                v-if="detailSkill.damage_calc"
                type="button"
                class="dom-calc-btn"
                :class="{ active: openSkillCalc === 'damage' }"
                :title="calcUiLabel(state)"
                @click.prevent.stop="toggleSkillCalc('damage')"
              >!</button>
            </span>
          </div>
          <div v-if="openSkillCalc === 'damage' && detailSkill.damage_calc" class="dom-calc-box">
            <div class="dom-calc-title">{{ calcUiLabel(state) }} · {{ labels.skill_damage || (uiLangFromState(state) === 'pt' ? 'Dano' : 'Damage') }}</div>
            <pre class="dom-calc-text">{{ trCalc(detailSkill.damage_calc) }}</pre>
          </div>

          <div v-if="detailSkill.range_calc" class="dom-status-row dom-status-row--calc">
            <span class="dom-status-key">{{ rangeUiLabel(state) }}</span>
            <span class="dom-status-val dom-status-val--with-calc">
              <span>{{ seeCalcUiLabel(state) }}</span>
              <button type="button" class="dom-calc-btn" :class="{ active: openSkillCalc === 'range' }" :title="calcUiLabel(state)" @click.prevent.stop="toggleSkillCalc('range')">!</button>
            </span>
          </div>
          <div v-if="detailSkill.range_calc && openSkillCalc === 'range'" class="dom-calc-box">
            <div class="dom-calc-title">{{ calcUiLabel(state) }} · {{ rangeUiLabel(state) }}</div>
            <pre class="dom-calc-text">{{ trCalc(detailSkill.range_calc) }}</pre>
          </div>

          <div v-if="detailSkill.knockback_calc" class="dom-status-row dom-status-row--calc">
            <span class="dom-status-key">{{ knockbackUiLabel(state) }}</span>
            <span class="dom-status-val dom-status-val--with-calc">
              <span>{{ seeCalcUiLabel(state) }}</span>
              <button type="button" class="dom-calc-btn" :class="{ active: openSkillCalc === 'knockback' }" :title="calcUiLabel(state)" @click.prevent.stop="toggleSkillCalc('knockback')">!</button>
            </span>
          </div>
          <div v-if="detailSkill.knockback_calc && openSkillCalc === 'knockback'" class="dom-calc-box">
            <div class="dom-calc-title">{{ calcUiLabel(state) }} · {{ knockbackUiLabel(state) }}</div>
            <pre class="dom-calc-text">{{ trCalc(detailSkill.knockback_calc) }}</pre>
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
