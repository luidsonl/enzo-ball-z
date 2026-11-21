<script setup>
defineProps({
  color: { type: String, required: true },
  selected: { type: Boolean, default: false },
  label: { type: String, default: "" },
});
defineEmits(["select"]);
</script>

<template>
  <button
    type="button"
    class="dom-swatch"
    :class="{ selected }"
    :style="{ background: color }"
    :aria-label="label"
    @click.prevent.stop="$emit('select')"
  />
</template>
