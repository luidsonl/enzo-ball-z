<script setup>
import { computed, watchEffect } from "vue";
import { uiState } from "./state/uiState.js";
import {
  applyDomInterfaceScalePercent,
  getEffectiveInterfaceScalePercent,
  isInterfaceScaleAuto,
  syncInterfaceScaleRuntime,
} from "./lib/mobile.js";
import MenuRoot from "./components/organisms/menu/MenuRoot.vue";
import HudRoot from "./components/organisms/hud/HudRoot.vue";
import MobileControls from "./components/organisms/controls/MobileControls.vue";

const state = computed(() => uiState.value);

watchEffect(() => {
  const percent = getEffectiveInterfaceScalePercent(state.value);
  const auto = state.value?.settings?.interface_scale_auto ?? isInterfaceScaleAuto();
  applyDomInterfaceScalePercent(percent);
  if (auto) syncInterfaceScaleRuntime(percent);
});
</script>

<template>
  <MenuRoot :state="state" />
  <HudRoot :state="state" />
  <MobileControls :state="state" />
</template>
