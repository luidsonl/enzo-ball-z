<script setup>
import PanelHeader from "../../../molecules/PanelHeader.vue";
import AppButton from "../../../atoms/AppButton.vue";

defineProps({
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
});
defineEmits(["close", "unequip"]);
</script>

<template>
  <PanelHeader
    :title="labels.equipment_title || labels.equipment || 'Equipment'"
    :close-label="labels.close || 'Close'"
    @close="$emit('close')"
  />
  <div class="dom-game-panel-body">
    <p v-if="!(player.equipment_slots || []).length" class="muted">{{ labels.no_item || '' }}</p>
    <div v-else class="dom-list">
      <div v-for="slot in (player.equipment_slots || [])" :key="slot.slot" class="dom-list-row">
        <span class="dom-list-key">{{ slot.slot_label || slot.slot }}</span>
        <span class="dom-list-val">{{ slot.item || labels.no_item || '—' }}</span>
        <AppButton
          v-if="slot.item_code != null"
          size="small"
          variant="secondary"
          style="margin-left:auto; pointer-events:auto; touch-action: manipulation"
          @click.prevent.stop="$emit('unequip', slot.slot_code)"
        >{{ labels.unequip || 'Unequip' }}</AppButton>
      </div>
    </div>
  </div>
</template>
