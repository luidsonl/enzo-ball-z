<script setup>
import AppButton from "../../atoms/AppButton.vue";
import AppTitle from "../../atoms/AppTitle.vue";

defineProps({
  phase: { type: String, default: "" },
  labels: { type: Object, default: () => ({}) },
});
defineEmits(["restart", "open-menu"]);
</script>

<template>
  <div class="dom-phase-overlay">
    <div class="dom-phase-panel">
      <AppTitle>{{ phase === 'Dead' ? (labels.game_over || 'Game Over') : (labels.victory || 'Victory') }}</AppTitle>
      <div class="dom-modal-actions">
        <AppButton variant="primary" @click.prevent.stop="$emit('restart')">{{ labels.restart || 'Restart' }}</AppButton>
        <AppButton variant="secondary" @click.prevent.stop="$emit('open-menu')">{{ labels.menu || 'Menu' }}</AppButton>
      </div>
    </div>
  </div>
</template>
