<script setup>
defineProps({
  value: { type: Number, default: 0 },
  max: { type: Number, default: 100 },
  variant: { type: String, default: "hp" }, // hp | aura | xp
  label: { type: String, default: "" },
});

function pct(v, m) {
  if (!m || m <= 0) return 0;
  return Math.max(0, Math.min(100, (v / m) * 100));
}
</script>

<template>
  <div class="dom-hud-bar-row">
    <span class="dom-hud-bar-label"><slot name="label">{{ label }}</slot></span>
    <div class="dom-hud-track" :class="variant">
      <div class="dom-hud-fill" :style="{ width: pct(value, max) + '%' }" />
    </div>
  </div>
</template>
