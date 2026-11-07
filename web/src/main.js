import { createApp } from "vue";
import App from "./App.vue";
import "./styles/index.css";
import {
  Action,
  MobileMode,
  UiAction,
  actionDown,
  actionUp,
  getMobileControlsMode,
  loadoutAssign,
  loadoutClear,
  uiAction,
} from "./bridge/bridge.js";
import { startStatePolling } from "./state/uiState.js";

createApp(App).mount("#dom-ui");
startStatePolling(150);

// Compatibility surface for web tests/debugging and external integrations.
globalThis.EnzoBridge = {
  Action,
  MOBILE_MODE: MobileMode,
  UiAction,
  actionDown,
  actionUp,
  uiAction,
  loadoutAssign,
  loadoutClear,
  getMobileControlsMode,
  // Layout helpers kept for compatibility; Vue controls manage rendering.
  getLayout() { return globalThis.__enzoMobileLayout?.get?.() ?? null; },
  setLayout(layout) { globalThis.__enzoMobileLayout?.set?.(layout); },
  resetLayout() { globalThis.__enzoMobileLayout?.reset?.(); },
  updateControl(id, patch) { globalThis.__enzoMobileLayout?.update?.(id, patch); },
  renderControls() { globalThis.__enzoMobileControls?.render?.(); },
  applyMobileControlsMode(state) { globalThis.__enzoMobileControls?.apply?.(state); },
};
