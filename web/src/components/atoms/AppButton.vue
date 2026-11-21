<script setup>
defineProps({
  variant: { type: String, default: "secondary" }, // primary | secondary | danger
  size: { type: String, default: "normal" }, // normal | small
  selected: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
  type: { type: String, default: "button" },
});
</script>

<template>
  <button
    :type="type"
    class="dom-btn"
    :class="[
      variant,
      { small: size === 'small', selected },
    ]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>
