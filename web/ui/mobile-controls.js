import { Action, MobileMode, actionDown, actionUp, getMobileControlsMode } from "./bridge.js";
import { subscribe } from "./state.js";

const LAYOUT_KEY = "enzoballz.mobile_layout.v1";

// Flexible, data-driven layout. `actions` are semantic InputAction codes.
// "Up" maps to Slot1 (jump) by default; it is not labeled.
const DEFAULT_LAYOUT = Object.freeze({
  version: 1,
  controls: [
    { id: "left",  label: "←", actions: [Action.MoveLeft],  x: 16,  y: 80, size: 64, anchor: "bottom-left" },
    { id: "down",  label: "↓", actions: [Action.MoveDown],  x: 88,  y: 16, size: 64, anchor: "bottom-left" },
    { id: "right", label: "→", actions: [Action.MoveRight], x: 160, y: 80, size: 64, anchor: "bottom-left" },
    { id: "up",    label: "↑", actions: [Action.Slot1],     x: 88,  y: 144,size: 64, anchor: "bottom-left" },

    { id: "six",   label: "6", actions: [Action.Six],       x: 16,  y: 224,size: 48, anchor: "bottom-left" },
    { id: "seven", label: "7", actions: [Action.Seven],     x: 72,  y: 224,size: 48, anchor: "bottom-left" },

    { id: "slot1", label: "1", actions: [Action.Slot1],     x: 160, y: 96, size: 60, anchor: "bottom-right", kind: "slot", slot: 1 },
    { id: "slot2", label: "2", actions: [Action.Slot2],     x: 88,  y: 96, size: 60, anchor: "bottom-right", kind: "slot", slot: 2 },
    { id: "slot3", label: "3", actions: [Action.Slot3],     x: 160, y: 24, size: 60, anchor: "bottom-right", kind: "slot", slot: 3 },
    { id: "slot4", label: "4", actions: [Action.Slot4],     x: 88,  y: 24, size: 60, anchor: "bottom-right", kind: "slot", slot: 4 },
  ],
});

function clone(obj) {
  return JSON.parse(JSON.stringify(obj));
}

function loadLayout() {
  try {
    const raw = localStorage.getItem(LAYOUT_KEY);
    if (!raw) return clone(DEFAULT_LAYOUT);
    const parsed = JSON.parse(raw);
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.controls)) {
      return clone(DEFAULT_LAYOUT);
    }
    return parsed;
  } catch (_) {
    return clone(DEFAULT_LAYOUT);
  }
}

function saveLayout(layout) {
  try {
    localStorage.setItem(LAYOUT_KEY, JSON.stringify(layout));
  } catch (_) {}
}

let currentLayout = loadLayout();

function applyAnchorStyle(el, control) {
  const size = Number(control.size || 56);
  el.style.width = size + "px";
  el.style.height = size + "px";
  el.style.position = "absolute";
  const x = Number(control.x || 0);
  const y = Number(control.y || 0);
  const anchor = control.anchor || "bottom-left";
  el.style.left = "auto";
  el.style.right = "auto";
  el.style.top = "auto";
  el.style.bottom = "auto";
  if (anchor === "bottom-left") {
    el.style.left = `calc(env(safe-area-inset-left) + ${x}px)`;
    el.style.bottom = `calc(env(safe-area-inset-bottom) + ${y}px)`;
  } else if (anchor === "bottom-right") {
    el.style.right = `calc(env(safe-area-inset-right) + ${x}px)`;
    el.style.bottom = `calc(env(safe-area-inset-bottom) + ${y}px)`;
  } else if (anchor === "top-left") {
    el.style.left = `calc(env(safe-area-inset-left) + ${x}px)`;
    el.style.top = `calc(env(safe-area-inset-top) + ${y}px)`;
  } else if (anchor === "top-right") {
    el.style.right = `calc(env(safe-area-inset-right) + ${x}px)`;
    el.style.top = `calc(env(safe-area-inset-top) + ${y}px)`;
  } else {
    el.style.left = x + "px";
    el.style.top = y + "px";
  }
}

function bindActions(el, actions) {
  const codes = Array.isArray(actions) ? actions.slice() : [];
  const activePointers = new Set();

  function press(e) {
    if (e.pointerType === "mouse" && e.button !== 0) return;
    activePointers.add(e.pointerId);
    try { el.setPointerCapture(e.pointerId); } catch (_) {}
    el.classList.add("active");
    for (const code of codes) actionDown(code);
    e.preventDefault();
    e.stopPropagation();
  }

  function release(e) {
    if (!activePointers.has(e.pointerId)) return;
    activePointers.delete(e.pointerId);
    if (activePointers.size === 0) {
      el.classList.remove("active");
      for (const code of codes) actionUp(code);
    }
    e.preventDefault();
    e.stopPropagation();
  }

  el.addEventListener("pointerdown", press, { passive: false });
  el.addEventListener("pointerup", release, { passive: false });
  el.addEventListener("pointercancel", release, { passive: false });
  el.addEventListener("pointerleave", release, { passive: false });
  el.addEventListener("contextmenu", function (e) { e.preventDefault(); });
}

export function renderControls() {
  const root = document.getElementById("dom-controls-root");
  if (!root) return;
  root.innerHTML = "";

  for (const control of (currentLayout.controls || [])) {
    if (!control || control.hidden) continue;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "ui-btn";
    btn.tabIndex = -1;
    btn.dataset.controlId = control.id || "";
    if (control.kind) btn.dataset.kind = control.kind;
    if (control.slot != null) btn.dataset.slot = String(control.slot);

    const label = document.createElement("span");
    label.className = "ui-label";
    label.textContent = control.label != null ? String(control.label) : "";
    btn.appendChild(label);

    applyAnchorStyle(btn, control);
    bindActions(btn, control.actions || []);
    root.appendChild(btn);
  }
}

export function setLayout(layout) {
  if (!layout || !Array.isArray(layout.controls)) return;
  currentLayout = layout;
  saveLayout(currentLayout);
  renderControls();
}

export function resetLayout() {
  setLayout(clone(DEFAULT_LAYOUT));
}

export function getLayout() {
  return clone(currentLayout);
}

export function updateControl(id, patch) {
  const layout = getLayout();
  const control = layout.controls.find((c) => c && c.id === id);
  if (!control) return;
  Object.assign(control, patch || {});
  setLayout(layout);
}

function isAutoMobile() {
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
  // auto / unknown: prefer live bridge mode, fallback to auto detection
  const mode = getMobileControlsMode();
  if (mode === MobileMode.On) return true;
  if (mode === MobileMode.Off) return false;
  return isAutoMobile();
}

export function applyMobileControlsMode(state) {
  const root = document.getElementById("dom-controls-root");
  if (!root) return;
  const visible = mobileControlsVisible(state?.settings, state?.scene);
  root.style.display = visible ? "" : "none";
  return visible;
}

function syncSlotControls(state) {
  const root = document.getElementById("dom-controls-root");
  if (!root) return;
  const slots = state?.player?.slots || [];
  root.querySelectorAll('button.ui-btn[data-kind="slot"]').forEach((btn) => {
    const layoutSlot = Number(btn.dataset.slot);
    if (!Number.isFinite(layoutSlot)) return;
    const slot = slots[layoutSlot - 1];
    const label = btn.querySelector(".ui-label");
    btn.classList.toggle("slot-active", !!slot?.active);
    btn.classList.toggle("slot-locked", !!(slot && !slot.available));
    btn.classList.toggle("slot-cooldown", !!(slot && slot.cooldown > 0.01));
    if (label) {
      const skill = slot?.skill != null ? `S${slot.skill}` : "";
      label.textContent = skill ? `${layoutSlot}\n${skill}` : String(layoutSlot);
      label.style.whiteSpace = "pre-line";
      label.style.fontSize = skill ? "12px" : "";
      label.style.textAlign = "center";
      label.style.lineHeight = "1.05";
    }
    let cd = btn.querySelector(".ui-slot-cd");
    if (slot && slot.cooldown > 0.01) {
      if (!cd) { cd = document.createElement("span"); cd.className = "ui-slot-cd"; btn.appendChild(cd); }
      cd.textContent = slot.cooldown.toFixed(1);
    } else if (cd) {
      cd.remove();
    }
  });
}

export function initMobileControls() {
  renderControls();
  applyMobileControlsMode();
  window.addEventListener("resize", () => applyMobileControlsMode());
  if (window.matchMedia) {
    const mq = window.matchMedia("(pointer: coarse)");
    if (mq && mq.addEventListener) mq.addEventListener("change", () => applyMobileControlsMode());
  }
  // React to settings/controller state (manual on/off/auto + slot visuals).
  subscribe((state) => {
    applyMobileControlsMode(state);
    syncSlotControls(state);
  });
}
