// DOM UI bridge for EnzoBallZ.
// Emits semantic InputAction codes to wasm exports and renders mobile controls
// from a flexible layout config (positions can be customized/persisted).
// Action codes must match enzo_core::input::InputAction::code() (append-only ABI).

(function () {
  "use strict";

  const Action = Object.freeze({
    MoveLeft: 0,
    MoveRight: 1,
    MoveUp: 2,
    MoveDown: 3,
    Slot1: 4,
    Slot2: 5,
    Slot3: 6,
    Slot4: 7,
    Six: 8,
    Seven: 9,
    Confirm: 14,
    Restart: 15,
    OpenMenu: 16,
    OpenSkills: 17,
  });

  const LAYOUT_KEY = "enzoballz.mobile_layout.v1";
  const MOBILE_MODE = Object.freeze({ Off: 0, On: 1, Auto: 2 });

  // Flexible, data-driven layout. `actions` are semantic InputAction codes sent
  // to the bridge, matching configured keyboard behavior.
  // Note: "Up" is mapped to Slot1 (jump) by default; it is not labeled.
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

  function exportsReady() {
    return typeof wasm_exports !== "undefined" && wasm_exports &&
      typeof wasm_exports.enzo_action_down === "function" &&
      typeof wasm_exports.enzo_action_up === "function";
  }

  function actionDown(code) {
    if (exportsReady()) wasm_exports.enzo_action_down(code);
  }

  function actionUp(code) {
    if (exportsReady()) wasm_exports.enzo_action_up(code);
  }

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

  function renderControls() {
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

      // Slot icon support can be added later (control.icon / UiState).
      const label = document.createElement("span");
      label.className = "ui-label";
      label.textContent = control.label != null ? String(control.label) : "";
      btn.appendChild(label);

      applyAnchorStyle(btn, control);
      bindActions(btn, control.actions || []);
      root.appendChild(btn);
    }
  }

  function setLayout(layout) {
    if (!layout || !Array.isArray(layout.controls)) return;
    currentLayout = layout;
    saveLayout(currentLayout);
    renderControls();
  }

  function resetLayout() {
    setLayout(clone(DEFAULT_LAYOUT));
  }

  function getLayout() {
    return clone(currentLayout);
  }

  function updateControl(id, patch) {
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

  function getMobileControlsMode() {
    try {
      if (exportsReady() && typeof wasm_exports.enzo_mobile_controls_mode === "function") {
        const mode = wasm_exports.enzo_mobile_controls_mode();
        if (mode === 0 || mode === 1 || mode === 2) return mode;
      }
    } catch (_) {}
    return MOBILE_MODE.Auto;
  }

  function applyMobileControlsMode() {
    const root = document.getElementById("dom-controls-root");
    if (!root) return;
    const mode = getMobileControlsMode();
    const visible = mode === MOBILE_MODE.On || (mode === MOBILE_MODE.Auto && isAutoMobile());
    root.style.display = visible ? "" : "none";
  }

  function pollUiState() {
    if (!exportsReady()) {
      applyMobileControlsMode();
      return;
    }
    applyMobileControlsMode();
    if (typeof wasm_exports.enzo_ui_state_ptr !== "function") return;
    try {
      const ptr = wasm_exports.enzo_ui_state_ptr();
      const len = wasm_exports.enzo_ui_state_len();
      if (!ptr || !len || typeof wasm_memory === "undefined" || !wasm_memory) return;
      const bytes = new Uint8Array(wasm_memory.buffer, ptr, len);
      const json = new TextDecoder().decode(bytes);
      if (!json) return;
      // Future: render DOM HUD and update slot icons from UiState.
      void json;
    } catch (_) {
      // Ignore bridge errors; canvas UI remains functional.
    }
  }

  function start() {
    renderControls();
    applyMobileControlsMode();
    window.addEventListener("resize", applyMobileControlsMode);
    if (window.matchMedia) {
      const mq = window.matchMedia("(pointer: coarse)");
      if (mq && mq.addEventListener) mq.addEventListener("change", applyMobileControlsMode);
    }
    setInterval(pollUiState, 250);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }

  window.EnzoBridge = {
    Action,
    MOBILE_MODE,
    actionDown,
    actionUp,
    getLayout,
    setLayout,
    resetLayout,
    updateControl,
    renderControls,
    applyMobileControlsMode,
    getMobileControlsMode,
  };
})();
