<script setup>
defineProps({
  muted: { type: Boolean, default: false },
});
</script>

<template>
  <span class="dom-keycap" :class="{ muted }"><slot /></span>
</template>
