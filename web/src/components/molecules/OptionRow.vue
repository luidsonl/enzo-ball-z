<script setup>
defineProps({
  label: { type: String, default: "" },
  danger: { type: Boolean, default: false },
});
</script>

<template>
  <div class="dom-option-row" :class="{ 'dom-option-row-danger': danger }">
    <div class="dom-option-label"><slot name="label">{{ label }}</slot></div>
    <div class="dom-option-buttons"><slot /></div>
  </div>
</template>
