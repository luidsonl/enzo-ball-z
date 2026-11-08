// Wasm bridge helpers (production). wasm_exports/wasm_memory are globals set by miniquad.
// Core rules remain authoritative in Rust; this layer only transports intents and reads UiState.

export const Action = Object.freeze({
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
  AllocateSpeed: 10,
  AllocateStrength: 11,
  AllocateHealth: 12,
  AllocateAura: 13,
  Confirm: 14,
  Restart: 15,
  OpenMenu: 16,
  OpenSkills: 17,
});

export const MobileMode = Object.freeze({ Off: 0, On: 1, Auto: 2 });

export const UiAction = Object.freeze({
  MenuBack: 1,
  MenuOpenSaves: 2,
  MenuOpenOptions: 3,
  MenuStart: 4,
  MenuResume: 5,
  MenuRequestDelete: 6,
  MenuCancelDelete: 7,
  MenuConfirmDelete: 8,
  SetLanguage: 9,
  SetMobileControls: 10,
  SetInterfaceScale: 11,
  HudAllocate: 12,
  HudConfirmLevelUp: 13,
  HudRestart: 14,
  HudOpenMenu: 15,
  HudOpenSkills: 16,
  EquipInventoryItem: 19,
  UnequipSlot: 20,
  SetPlayableCharacter: 21,
  SetVisualProfileField: 22,
  MenuOpenCustomize: 23,
});

const pendingUi = [];
const textDecoder = new TextDecoder();

export function exportsReady() {
  return typeof globalThis.wasm_exports !== "undefined" && globalThis.wasm_exports &&
    typeof globalThis.wasm_exports.enzo_action_down === "function" &&
    typeof globalThis.wasm_exports.enzo_action_up === "function";
}

export function flushPending() {
  if (!exportsReady() || typeof globalThis.wasm_exports.enzo_ui_action !== "function") return;
  while (pendingUi.length) {
    const item = pendingUi.shift();
    try {
      if (item.kind === "ui") globalThis.wasm_exports.enzo_ui_action(item.code, item.a, item.b, item.c);
      else if (item.kind === "assign" && globalThis.wasm_exports.enzo_loadout_assign) {
        globalThis.wasm_exports.enzo_loadout_assign(item.slot, item.skill);
      } else if (item.kind === "clear" && globalThis.wasm_exports.enzo_loadout_clear) {
        globalThis.wasm_exports.enzo_loadout_clear(item.slot);
      }
    } catch (_) {}
  }
}

export function actionDown(code) {
  if (exportsReady()) globalThis.wasm_exports.enzo_action_down(code);
}

export function actionUp(code) {
  if (exportsReady()) globalThis.wasm_exports.enzo_action_up(code);
}

export function uiAction(code, a = 0, b = 0, c = 0) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_ui_action === "function") {
    globalThis.wasm_exports.enzo_ui_action(code, a, b, c);
    return;
  }
  if (pendingUi.length < 64) pendingUi.push({ kind: "ui", code, a, b, c });
}

export function loadoutAssign(slot, skill) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_loadout_assign === "function") {
    globalThis.wasm_exports.enzo_loadout_assign(slot, skill);
    return;
  }
  if (pendingUi.length < 64) pendingUi.push({ kind: "assign", slot, skill });
}

export function loadoutClear(slot) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_loadout_clear === "function") {
    globalThis.wasm_exports.enzo_loadout_clear(slot);
    return;
  }
  if (pendingUi.length < 64) pendingUi.push({ kind: "clear", slot });
}

export function equipItem(itemCode) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_equip_item === "function") {
    globalThis.wasm_exports.enzo_equip_item(itemCode);
    return;
  }
  uiAction(UiAction.EquipInventoryItem, itemCode, 0, 0);
}

export function unequipSlot(slotCode) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_unequip_slot === "function") {
    globalThis.wasm_exports.enzo_unequip_slot(slotCode);
    return;
  }
  uiAction(UiAction.UnequipSlot, slotCode, 0, 0);
}

export function setPlayableCharacter(characterCode) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_set_character === "function") {
    globalThis.wasm_exports.enzo_set_character(characterCode);
    return;
  }
  uiAction(UiAction.SetPlayableCharacter, characterCode, 0, 0);
}

export function setVisualProfileField(field, value) {
  if (exportsReady() && typeof globalThis.wasm_exports.enzo_set_visual_profile_field === "function") {
    globalThis.wasm_exports.enzo_set_visual_profile_field(field, value);
    return;
  }
  uiAction(UiAction.SetVisualProfileField, field, value, 0);
}

export function getMobileControlsMode() {
  try {
    if (exportsReady() && typeof globalThis.wasm_exports.enzo_mobile_controls_mode === "function") {
      const mode = globalThis.wasm_exports.enzo_mobile_controls_mode();
      if (mode === 0 || mode === 1 || mode === 2) return mode;
    }
  } catch (_) {}
  return MobileMode.Auto;
}

export function readUiStateJson() {
  try {
    if (!exportsReady() || typeof globalThis.wasm_exports.enzo_ui_state_ptr !== "function") return null;
    const ptr = globalThis.wasm_exports.enzo_ui_state_ptr();
    const len = globalThis.wasm_exports.enzo_ui_state_len();
    if (!ptr || !len || typeof globalThis.wasm_memory === "undefined" || !globalThis.wasm_memory) return null;
    const bytes = new Uint8Array(globalThis.wasm_memory.buffer, ptr, len);
    const json = textDecoder.decode(bytes);
    return json || null;
  } catch (_) {
    return null;
  }
}
