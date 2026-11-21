<script setup>
defineProps({
  show: { type: Boolean, default: true },
  overlayClass: { type: [String, Array, Object], default: "" },
  modalClass: { type: [String, Array, Object], default: "" },
});
defineEmits(["close"]);
</script>

<template>
  <div v-if="show" class="dom-modal-overlay" :class="overlayClass" @click.self="$emit('close')">
    <div class="dom-modal" :class="modalClass" @click.stop>
      <slot />
    </div>
  </div>
</template>
