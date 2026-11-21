<script setup>
import AppButton from "../atoms/AppButton.vue";

defineProps({
  slotData: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
});
defineEmits(["load", "delete", "new-game"]);
</script>

<template>
  <div class="dom-slot-card">
    <div class="dom-slot-title">{{ labels.slot }} {{ slotData.slot + 1 }}</div>
    <template v-if="slotData.occupied">
      <div class="dom-slot-meta">
        {{ labels.level }} {{ slotData.level }} · {{ Math.floor(slotData.xp) }}/{{ Math.floor(slotData.xp_to_next) }} XP · {{ slotData.points }} {{ labels.points }}
      </div>
      <div class="dom-slot-actions">
        <AppButton variant="primary" size="small" @click.prevent.stop="$emit('load')">{{ labels.load }}</AppButton>
        <AppButton variant="danger" size="small" @click.prevent.stop="$emit('delete')">{{ labels.delete }}</AppButton>
      </div>
    </template>
    <template v-else>
      <div class="dom-slot-meta muted">{{ labels.empty_slot }}</div>
      <div class="dom-slot-actions">
        <AppButton variant="primary" size="small" @click.prevent.stop="$emit('new-game')">{{ labels.new_game }}</AppButton>
      </div>
    </template>
  </div>
</template>
