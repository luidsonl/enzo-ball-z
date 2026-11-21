<script setup>
import ProgressBar from "../atoms/ProgressBar.vue";

defineProps({
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
});
</script>

<template>
  <div class="dom-hud-bars">
    <ProgressBar variant="hp" :value="player.health" :max="player.max_health">
      <template #label>
        HP {{ Math.ceil(player.health) }}/{{ Math.ceil(player.max_health) }}
        <span v-if="player.sigma_mode" style="margin-left:8px; color:#ffd966; font-weight:700">{{ labels.sigma_mode || 'Sigma' }}</span>
      </template>
    </ProgressBar>
    <ProgressBar variant="aura" :value="player.aura" :max="player.max_aura">
      <template #label>Aura {{ Math.ceil(player.aura) }}/{{ Math.ceil(player.max_aura) }}</template>
    </ProgressBar>
    <ProgressBar variant="xp" :value="player.xp" :max="player.xp_to_next">
      <template #label>Lv {{ player.level }} {{ Math.floor(player.xp) }}/{{ Math.floor(player.xp_to_next) }}</template>
    </ProgressBar>
  </div>
</template>
