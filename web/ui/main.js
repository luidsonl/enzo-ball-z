// Modular DOM UI entry point.

import { Action, MobileMode, UiAction, actionDown, actionUp, loadoutAssign, loadoutClear, uiAction } from "./bridge.js";
import {
  applyMobileControlsMode,
  getLayout,
  initMobileControls,
  renderControls,
  resetLayout,
  setLayout,
  updateControl,
} from "./mobile-controls.js";
import { initHud } from "./hud.js";
import { initMenus } from "./menus.js";
import { initState, subscribe } from "./state.js";

function applyInterfaceScale(state) {
  const scale = state?.settings?.interface_scale;
  const value = typeof scale === "number" && isFinite(scale) ? Math.max(0.5, Math.min(3, scale)) : 1;
  document.documentElement.style.setProperty("--dom-ui-scale", String(value));
  const domUi = document.getElementById("dom-ui");
  if (domUi) {
    // Controls the new DOM UI scale (menus/HUD/overlays). Canvas game remains separate.
    domUi.style.zoom = String(value);
  }
}

function start() {
  initMobileControls();
  initState();
  initHud();
  initMenus();
  subscribe(applyInterfaceScale);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start);
} else {
  start();
}

globalThis.EnzoBridge = {
  Action,
  MOBILE_MODE: MobileMode,
  UiAction,
  actionDown,
  actionUp,
  uiAction,
  loadoutAssign,
  loadoutClear,
  getLayout,
  setLayout,
  resetLayout,
  updateControl,
  renderControls,
  applyMobileControlsMode,
};
