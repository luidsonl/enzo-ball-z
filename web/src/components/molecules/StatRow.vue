<script setup>
defineProps({
  label: { type: String, default: "" },
  value: { type: [String, Number], default: "" },
});
</script>

<template>
  <div class="dom-status-row">
    <span class="dom-status-key"><slot name="label">{{ label }}</slot></span>
    <span class="dom-status-val"><slot>{{ value }}</slot></span>
  </div>
</template>
