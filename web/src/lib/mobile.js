import { MobileMode, getMobileControlsMode } from "../bridge/bridge.js";

export const INTERFACE_SCALE_STORAGE_KEY = "enzoballz.interface_scale";
export const INTERFACE_SCALE_AUTO_STORAGE_KEY = "enzoballz.interface_scale_auto";

export function isAutoMobile() {
  if (typeof window === "undefined") return false;
  const coarse = window.matchMedia && window.matchMedia("(pointer: coarse)").matches;
  const touch = (navigator.maxTouchPoints || 0) > 0;
  const small = Math.min(window.innerWidth, window.innerHeight) <= 900;
  return coarse || (touch && small);
}

export function getAutoInterfaceScalePercent() {
  return isAutoMobile() ? 100 : 125;
}

export function readStoredInterfaceScale() {
  try {
    if (typeof window === "undefined" || !window.localStorage) return null;
    const raw = window.localStorage.getItem(INTERFACE_SCALE_STORAGE_KEY);
    if (raw == null || raw === "") return null;
    const value = Number(raw);
    return Number.isFinite(value) && value >= 0.75 && value <= 2.0 ? value : null;
  } catch (_) {
    return null;
  }
}

export function isInterfaceScaleAuto() {
  try {
    if (typeof window === "undefined" || !window.localStorage) return readStoredInterfaceScale() == null;
    const raw = window.localStorage.getItem(INTERFACE_SCALE_AUTO_STORAGE_KEY);
    if (raw === "1" || raw === "true" || raw === "on") return true;
    if (raw === "0" || raw === "false" || raw === "off") return false;
    // Migration/default: auto unless a manual scale was explicitly stored previously.
    return readStoredInterfaceScale() == null;
  } catch (_) {
    return true;
  }
}

export function getEffectiveInterfaceScalePercent(state = null) {
  const auto = state?.settings?.interface_scale_auto ?? isInterfaceScaleAuto();
  if (auto) return getAutoInterfaceScalePercent();
  const fromState = state?.settings?.interface_scale;
  if (typeof fromState === "number" && Number.isFinite(fromState)) return Math.round(fromState * 100);
  const stored = readStoredInterfaceScale();
  return stored != null ? Math.round(stored * 100) : 125;
}

export function applyDomInterfaceScalePercent(percent) {
  const value = String(Math.max(50, Math.min(300, percent)) / 100);
  try {
    if (typeof document !== "undefined") {
      document.documentElement.style.setProperty("--dom-ui-scale", value);
      const domUi = document.getElementById("dom-ui");
      if (domUi) domUi.style.zoom = value;
    }
  } catch (_) {}
}

export function syncInterfaceScaleRuntime(percent) {
  const applyRuntime = () => {
    try {
      if (typeof globalThis.wasm_exports?.enzo_set_interface_scale_runtime === "function") {
        globalThis.wasm_exports.enzo_set_interface_scale_runtime(Math.round(percent));
        return true;
      }
    } catch (_) {}
    return false;
  };

  if (applyRuntime()) return true;

  let attempts = 0;
  const timer = setInterval(() => {
    attempts += 1;
    if (applyRuntime() || attempts > 120) clearInterval(timer);
  }, 100);
  return false;
}

export function syncInterfaceScale(state = null) {
  const percent = getEffectiveInterfaceScalePercent(state);
  const auto = state?.settings?.interface_scale_auto ?? isInterfaceScaleAuto();
  if (auto) {
    applyDomInterfaceScalePercent(percent);
    syncInterfaceScaleRuntime(percent);
  }
  return percent;
}

export function mobileControlsVisible(settings, scene) {
  if (scene === "menu" || scene === "menu_over_game") return false;
  const modeCode = settings?.mobile_controls;
  if (modeCode === "on") return true;
  if (modeCode === "off") return false;
  const mode = getMobileControlsMode();
  if (mode === MobileMode.On) return true;
  if (mode === MobileMode.Off) return false;
  return isAutoMobile();
}
