<script setup>
import { computed, watchEffect } from "vue";
import { uiState } from "./state/uiState.js";
import MenuRoot from "./components/menus/MenuRoot.vue";
import HudRoot from "./components/hud/HudRoot.vue";
import MobileControls from "./components/controls/MobileControls.vue";

const state = computed(() => uiState.value);

watchEffect(() => {
  const scale = state.value?.settings?.interface_scale;
  const value = typeof scale === "number" && Number.isFinite(scale) ? Math.max(0.5, Math.min(3, scale)) : 1;
  document.documentElement.style.setProperty("--dom-ui-scale", String(value));
  const domUi = document.getElementById("dom-ui");
  if (domUi) domUi.style.zoom = String(value);
});
</script>

<template>
  <MenuRoot :state="state" />
  <HudRoot :state="state" />
  <MobileControls :state="state" />
</template>
