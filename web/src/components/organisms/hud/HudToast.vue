<script setup>
defineProps({
  message: { type: String, default: "" },
});
</script>

<template>
  <div v-if="message" class="dom-game-toast">{{ message }}</div>
</template>
