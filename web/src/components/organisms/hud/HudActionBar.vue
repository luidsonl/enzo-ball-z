<script setup>
import ActionSlotButton from "../../molecules/ActionSlotButton.vue";

defineProps({
  slots: { type: Array, default: () => [] },
  slotKeyLabel: { type: Function, required: true },
});
defineEmits(["open-skills"]);
</script>

<template>
  <div class="dom-action-bar" style="pointer-events: auto">
    <ActionSlotButton
      v-for="(slot, index) in slots"
      :key="index"
      :slot="slot"
      :index="index"
      :key-label="slotKeyLabel(index)"
      @select="$emit('open-skills')"
    />
  </div>
</template>
