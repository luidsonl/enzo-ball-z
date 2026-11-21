<script setup>
import PanelHeader from "../../../molecules/PanelHeader.vue";
import StatRow from "../../../molecules/StatRow.vue";
import AppButton from "../../../atoms/AppButton.vue";

defineProps({
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
  attrDefs: { type: Array, default: () => [] }, // [name,value,code]
});
defineEmits(["close", "allocate"]);
</script>

<template>
  <PanelHeader
    :title="labels.status_title || labels.status || 'Status'"
    :close-label="labels.close || 'Close'"
    @close="$emit('close')"
  />
  <div class="dom-game-panel-body">
    <div class="dom-status-grid">
      <StatRow :label="labels.level || 'Level'" :value="player.level" />
      <StatRow label="XP" :value="`${Math.floor(player.xp)} / ${Math.floor(player.xp_to_next)}`" />
      <StatRow :label="labels.points || 'Points'" :value="player.points" />
      <StatRow label="HP" :value="`${Math.ceil(player.health)} / ${Math.ceil(player.max_health)}`" />
      <StatRow label="Aura" :value="`${Math.ceil(player.aura)} / ${Math.ceil(player.max_aura)}`" />
    </div>

    <h4 class="dom-subtitle">{{ labels.points || 'Points' }}</h4>
    <template v-if="(player.points || 0) > 0">
      <p class="muted">{{ labels.points || 'Points' }}: {{ player.points }}</p>
      <div class="dom-attr-grid">
        <AppButton
          v-for="[name, value, code] in attrDefs"
          :key="code"
          size="small"
          variant="secondary"
          style="pointer-events:auto; touch-action: manipulation"
          @click.prevent.stop="$emit('allocate', code)"
        >+ {{ name }} ({{ value }})</AppButton>
      </div>
    </template>
    <div v-else class="dom-status-grid">
      <StatRow v-for="[name, value] in attrDefs" :key="name" :label="name" :value="value" />
    </div>
  </div>
</template>
