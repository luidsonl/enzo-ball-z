<script setup>
import { computed } from "vue";
import PanelHeader from "../../../molecules/PanelHeader.vue";
import AppButton from "../../../atoms/AppButton.vue";

const props = defineProps({
  state: { type: Object, default: null },
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
  sandbox: { type: Object, required: true },
});
const emit = defineEmits([
  "close",
  "set-tab",
  "select-block",
  "set-enemy",
  "set-item",
  "toggle-sigma",
  "grant-levels",
  "set-level",
  "reset-attrs",
  "die",
  "reset-sandbox",
]);

const sandboxTab = computed(() => props.sandbox?.tab ?? 0);
const sandboxBlocks = computed(() => props.sandbox?.blocks || []);
const sandboxEnemies = computed(() => props.sandbox?.enemies || []);
const sandboxItems = computed(() => props.sandbox?.items || []);
function uiLang() {
  const lang = props.state?.settings?.language || "pt-BR";
  return String(lang).toLowerCase().startsWith("en") ? "en" : "pt";
}
</script>

<template>
  <PanelHeader
    :title="labels.sandbox_debug || 'Debug'"
    :close-label="labels.close || 'Close'"
    @close="$emit('close')"
  />
  <div class="dom-game-panel-body">
    <div class="dom-slot-picker" style="margin-bottom:10px; flex-wrap:wrap">
      <AppButton size="small" variant="secondary" class="dom-slot-pick-btn" :selected="sandboxTab === 0" @click.prevent.stop="$emit('set-tab', 0)">{{ labels.tab_character || 'Character' }}</AppButton>
      <AppButton size="small" variant="secondary" class="dom-slot-pick-btn" :selected="sandboxTab === 1" @click.prevent.stop="$emit('set-tab', 1)">{{ labels.tab_blocks || 'Blocks' }}</AppButton>
      <AppButton size="small" variant="secondary" class="dom-slot-pick-btn" :selected="sandboxTab === 2" @click.prevent.stop="$emit('set-tab', 2)">{{ labels.tab_enemies || 'Enemies' }}</AppButton>
      <AppButton size="small" variant="secondary" class="dom-slot-pick-btn" :selected="sandboxTab === 3" @click.prevent.stop="$emit('set-tab', 3)">{{ labels.tab_items || 'Items' }}</AppButton>
    </div>

    <template v-if="sandboxTab === 0">
      <div class="dom-status-grid">
        <div class="dom-status-row"><span class="dom-status-key">{{ labels.level || 'Level' }}</span><span class="dom-status-val">{{ player.level }}</span></div>
        <div class="dom-status-row"><span class="dom-status-key">{{ labels.points || 'Points' }}</span><span class="dom-status-val">{{ player.points }}</span></div>
        <div class="dom-status-row"><span class="dom-status-key">{{ labels.sigma_mode || 'Sigma mode' }}</span><span class="dom-status-val">{{ player.sigma_mode ? (labels.on || 'On') : (labels.off || 'Off') }}</span></div>
      </div>
      <div class="dom-option-row" style="margin-top:10px; display:flex; gap:8px; flex-wrap:wrap;">
        <AppButton size="small" :variant="player.sigma_mode ? 'primary' : 'secondary'" @click.prevent.stop="$emit('toggle-sigma')">{{ labels.sigma_mode || 'Sigma mode' }}</AppButton>
        <AppButton size="small" variant="primary" @click.prevent.stop="$emit('grant-levels', 1)">{{ labels.level_up_debug || 'Level up' }} +1</AppButton>
        <AppButton size="small" variant="primary" @click.prevent.stop="$emit('grant-levels', 5)">+5</AppButton>
        <AppButton size="small" variant="secondary" @click.prevent.stop="$emit('set-level', 1)">Lv 1</AppButton>
        <AppButton size="small" variant="secondary" @click.prevent.stop="$emit('set-level', 10)">Lv 10</AppButton>
      </div>
      <div class="dom-option-row" style="margin-top:8px; display:flex; gap:8px; flex-wrap:wrap;">
        <AppButton size="small" variant="secondary" @click.prevent.stop="$emit('reset-attrs')">{{ labels.reset_attrs || 'Reset attributes' }}</AppButton>
        <AppButton size="small" variant="danger" @click.prevent.stop="$emit('die')">{{ labels.die || 'Die' }}</AppButton>
        <AppButton size="small" variant="danger" @click.prevent.stop="$emit('reset-sandbox')">{{ labels.reset_sandbox || 'Reset sandbox' }}</AppButton>
      </div>
    </template>

    <template v-else-if="sandboxTab === 1">
      <p class="muted" style="margin-bottom:8px">{{ uiLang() === 'pt' ? 'Selecione um bloco ou a borracha e clique no mapa.' : 'Select a block or the eraser and click the map.' }}</p>
      <div class="dom-debug-grid">
        <button
          v-for="b in sandboxBlocks"
          :key="'b'+b.code+'-'+(b.remove?1:0)"
          type="button"
          class="dom-debug-slot"
          :class="{
            selected: b.remove ? (sandbox.block_mode ?? 0) === 1 : ((sandbox.block_mode ?? 0) === 0 && sandbox.selected_block === b.code),
            remove: !!b.remove
          }"
          :title="b.remove ? (labels.remove || 'Remove') : b.name"
          @click.prevent.stop="b.remove ? $emit('select-block', sandbox.selected_block ?? 2, 1) : $emit('select-block', b.code, 0)"
        >
          <span v-if="b.remove" class="dom-debug-slot-icon dom-debug-remove-icon" aria-hidden="true">✕</span>
          <img v-else-if="b.icon" class="dom-debug-slot-icon pixel" :src="b.icon" :alt="b.name" />
          <span v-else class="dom-debug-slot-fallback">{{ b.name.slice(0, 1) }}</span>
          <span class="dom-debug-slot-name">{{ b.remove ? (labels.remove || 'Remove') : b.name }}</span>
        </button>
      </div>
    </template>

    <template v-else-if="sandboxTab === 2">
      <p class="muted" style="margin-bottom:8px">{{ uiLang() === 'pt' ? 'Selecione um inimigo e clique no mapa para spawnar.' : 'Select an enemy and click the map to spawn it.' }}</p>
      <div class="dom-debug-grid">
        <button
          v-for="e in sandboxEnemies"
          :key="'e'+e.code"
          type="button"
          class="dom-debug-slot"
          :class="{ selected: sandbox.selected_enemy === e.code }"
          :title="e.name"
          @click.prevent.stop="$emit('set-enemy', e.code)"
        >
          <span
            class="dom-debug-slot-icon dom-debug-sprite"
            :style="{
              backgroundImage: `url(${e.sprite})`,
              backgroundPosition: `-${e.frame_x}px -${e.frame_y}px`,
              backgroundSize: `${e.sheet_w}px ${e.sheet_h}px`,
              width: `${e.frame_w}px`,
              height: `${e.frame_h}px`,
            }"
            aria-hidden="true"
          ></span>
          <span class="dom-debug-slot-name">{{ e.name }}</span>
        </button>
      </div>
    </template>

    <template v-else-if="sandboxTab === 3">
      <p class="muted" style="margin-bottom:8px">{{ uiLang() === 'pt' ? 'Selecione um item e clique no mapa para spawnar.' : 'Select an item and click the map to spawn it.' }}</p>
      <div class="dom-debug-grid">
        <button
          v-for="it in sandboxItems"
          :key="'i'+it.code"
          type="button"
          class="dom-debug-slot"
          :class="{ selected: sandbox.selected_item === it.code }"
          :title="it.name"
          @click.prevent.stop="$emit('set-item', it.code)"
        >
          <img v-if="it.icon" class="dom-debug-slot-icon pixel" :src="it.icon" :alt="it.name" />
          <span v-else class="dom-debug-slot-fallback">?</span>
          <span class="dom-debug-slot-name">{{ it.name }}</span>
        </button>
      </div>
    </template>
  </div>
</template>
