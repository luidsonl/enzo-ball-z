<script setup>
defineProps({
  tag: { type: String, default: "h2" },
});
</script>

<template>
  <component :is="tag" class="dom-title">
    <slot />
  </component>
</template>
