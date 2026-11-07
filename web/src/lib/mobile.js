import { MobileMode, getMobileControlsMode } from "../bridge/bridge.js";

export function isAutoMobile() {
  if (typeof window === "undefined") return false;
  const coarse = window.matchMedia && window.matchMedia("(pointer: coarse)").matches;
  const touch = (navigator.maxTouchPoints || 0) > 0;
  const small = Math.min(window.innerWidth, window.innerHeight) <= 900;
  return coarse || (touch && small);
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
