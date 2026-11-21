<script setup>
defineProps({
  slot: { type: Object, required: true },
  index: { type: Number, required: true },
  keyLabel: { type: String, default: "" },
});
defineEmits(["select"]);
</script>

<template>
  <button
    type="button"
    class="dom-action-slot"
    :class="{ active: slot.active, locked: !slot.available, cooldown: slot.cooldown > 0.01 }"
    :title="slot.lock_reason || ''"
    @click.prevent.stop="$emit('select')"
  >
    <span class="slot-num">{{ keyLabel || String(index + 1) }}</span>
    <img v-if="slot.skill_icon" class="slot-skill-icon" :src="slot.skill_icon" :alt="slot.skill_name || ''" :title="slot.skill_name || ''" />
    <span v-else class="slot-skill">—</span>
    <span v-if="slot.cooldown > 0.01" class="slot-cd">{{ slot.cooldown.toFixed(1) }}</span>
  </button>
</template>
