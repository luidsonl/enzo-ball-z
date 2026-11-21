<script setup>
import HudTopBars from "../../molecules/HudTopBars.vue";

defineProps({
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
  tabs: { type: Array, default: () => [] }, // [id,label,blink][]
  openPanel: { type: String, default: null },
  fullscreen: { type: Boolean, default: false },
  fullscreenLabel: { type: String, default: "Fullscreen" },
  menuLabel: { type: String, default: "Menu" },
  settings: { type: Object, default: () => ({}) },
});
defineEmits(["toggle-panel", "toggle-fullscreen", "open-menu"]);
</script>

<template>
  <div class="dom-hud-top">
    <HudTopBars :player="player" :labels="labels" />

    <div class="dom-hud-tabs">
      <button
        v-for="[id, label, blink] in tabs"
        :key="id"
        type="button"
        class="dom-hud-tab"
        :class="{ active: openPanel === id, blink }"
        @click.prevent.stop="$emit('toggle-panel', id)"
      >{{ label }}</button>
    </div>

    <div class="dom-hud-actions" style="pointer-events: auto">
      <button
        type="button"
        class="dom-hud-btn dom-hud-fullscreen-btn"
        :data-lang="settings?.language || 'en'"
        :title="fullscreenLabel"
        :aria-pressed="fullscreen ? 'true' : 'false'"
        @click.prevent.stop="$emit('toggle-fullscreen')"
      >{{ fullscreenLabel }}</button>
      <button
        type="button"
        class="dom-hud-btn dom-hud-menu-btn"
        @click.prevent.stop="$emit('open-menu')"
      >{{ menuLabel }}</button>
    </div>
  </div>
</template>
