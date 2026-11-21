<script setup>
import AppButton from "../atoms/AppButton.vue";
import AppTitle from "../atoms/AppTitle.vue";

defineProps({
  title: { type: String, default: "" },
  closeLabel: { type: String, default: "Close" },
});
defineEmits(["close"]);
</script>

<template>
  <div class="dom-game-panel-header">
    <AppTitle tag="h3">{{ title }}</AppTitle>
    <AppButton size="small" variant="secondary" @click.prevent.stop="$emit('close')">
      {{ closeLabel }}
    </AppButton>
  </div>
</template>
