<script setup>
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { Action, actionDown, actionUp } from "../../bridge/bridge.js";
import { mobileControlsVisible } from "../../lib/mobile.js";

const props = defineProps({
  state: { type: Object, default: null },
});

const LAYOUT_KEY = "enzoballz.mobile_layout.v1";
const DEFAULT_LAYOUT = {
  version: 1,
  controls: [
    { id: "joystick", kind: "joystick", x: 32, y: 32, size: 148, anchor: "bottom-left" },
    { id: "six",   label: "6", actions: [Action.Six],       x: 16,  y: 224, size: 48, anchor: "bottom-left" },
    { id: "seven", label: "7", actions: [Action.Seven],     x: 72,  y: 224, size: 48, anchor: "bottom-left" },
    { id: "slot1", label: "1", actions: [Action.Slot1],     x: 90, y: 96,  size: 60, anchor: "bottom-right", kind: "slot", slot: 1 },
    { id: "slot2", label: "2", actions: [Action.Slot2],     x: 18,  y: 96,  size: 60, anchor: "bottom-right", kind: "slot", slot: 2 },
    { id: "slot3", label: "3", actions: [Action.Slot3],     x: 90, y: 24,  size: 60, anchor: "bottom-right", kind: "slot", slot: 3 },
    { id: "slot4", label: "4", actions: [Action.Slot4],     x: 18,  y: 24,  size: 60, anchor: "bottom-right", kind: "slot", slot: 4 },
  ],
};

const DIR_BUTTON_IDS = new Set(["left", "right", "down"]);
const MOVE_ACTIONS = [Action.MoveLeft, Action.MoveRight, Action.MoveUp, Action.MoveDown];

function isLegacyUpSlotButton(control) {
  if (!control || control.kind === "joystick" || control.kind === "slot") return false;
  if (control.id !== "up") return false;
  const actions = Array.isArray(control.actions) ? control.actions : [];
  return actions.length === 1 && actions[0] === Action.Slot1;
}

function clone(obj) { return JSON.parse(JSON.stringify(obj)); }
function loadLayout() {
  try {
    const raw = localStorage.getItem(LAYOUT_KEY);
    if (!raw) return clone(DEFAULT_LAYOUT);
    const parsed = JSON.parse(raw);
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.controls)) return clone(DEFAULT_LAYOUT);
    return parsed;
  } catch { return clone(DEFAULT_LAYOUT); }
}
function saveLayout(layout) {
  try { localStorage.setItem(LAYOUT_KEY, JSON.stringify(layout)); } catch {}
}

const layout = ref(loadLayout());
const activePointers = reactive(new Map()); // controlId -> Set(pointerId)
const activeClasses = reactive({});

const visible = computed(() => mobileControlsVisible(props.state?.settings, props.state?.scene));
const slots = computed(() => props.state?.player?.slots || []);

function anchorClass(anchor) {
  if (anchor === "bottom-right" || anchor === "top-left" || anchor === "top-right") return `anchor-${anchor}`;
  return "anchor-bottom-left";
}

function layoutVars(control) {
  return {
    "--control-size": `${Number(control.size || 56)}px`,
    "--control-x": `${Number(control.x || 0)}px`,
    "--control-y": `${Number(control.y || 0)}px`,
  };
}

function isDirectionalMovementButton(control) {
  if (!control || control.hidden || control.kind === "slot" || control.kind === "joystick") return false;
  if (DIR_BUTTON_IDS.has(control.id)) return true;
  const actions = Array.isArray(control.actions) ? control.actions : [];
  return actions.length > 0 && actions.every((a) => a === Action.MoveLeft || a === Action.MoveRight || a === Action.MoveUp || a === Action.MoveDown);
}

const renderControls = computed(() => {
  const controls = (layout.value.controls || []).filter((c) => c && !c.hidden && !isLegacyUpSlotButton(c));
  if (controls.some((c) => c.kind === "joystick")) return controls;
  const dir = controls.filter(isDirectionalMovementButton);
  if (!dir.length) return controls;
  const others = controls.filter((c) => !isDirectionalMovementButton(c));

  const xs = dir.map((c) => Number(c.x || 0));
  const ys = dir.map((c) => Number(c.y || 0));
  const sizes = dir.map((c) => Number(c.size || 56));
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  const maxX = Math.max(...xs.map((x, i) => x + sizes[i]));
  const maxY = Math.max(...ys.map((y, i) => y + sizes[i]));
  const span = Math.max(140, maxX - minX, maxY - minY);

  others.push({
    id: "__joystick_auto",
    kind: "joystick",
    anchor: dir[0].anchor || "bottom-left",
    x: Math.max(0, minX - 8),
    y: Math.max(0, minY - 8),
    size: Math.min(180, span + 16),
    __auto: true,
  });
  return others;
});

function ensurePointerSet(id) {
  if (!activePointers.has(id)) activePointers.set(id, new Set());
  return activePointers.get(id);
}

function press(control, e) {
  if (e.pointerType === "mouse" && e.button !== 0) return;
  const set = ensurePointerSet(control.id);
  set.add(e.pointerId);
  try { e.currentTarget?.setPointerCapture?.(e.pointerId); } catch {}
  activeClasses[control.id] = true;
  for (const code of (control.actions || [])) actionDown(code);
  e.preventDefault();
  e.stopPropagation();
}

function release(control, e) {
  const set = activePointers.get(control.id);
  if (!set || !set.has(e.pointerId)) return;
  set.delete(e.pointerId);
  if (set.size === 0) {
    activeClasses[control.id] = false;
    for (const code of (control.actions || [])) actionUp(code);
  }
  e.preventDefault();
  e.stopPropagation();
}

function slotInfo(control) {
  if (control.kind !== "slot" || control.slot == null) return null;
  return slots.value[control.slot - 1] || null;
}

function slotSkillName(slot) {
  const skillName = slot?.skill_name || (slot?.skill != null ? `S${slot.skill}` : "");
  return skillName.length > 8 ? skillName.slice(0, 7) + "…" : skillName;
}

function slotClasses(control) {
  const slot = slotInfo(control);
  if (!slot) return {};
  return {
    "ui-btn--slot": true,
    "has-skill": !!(slot.skill_name || slot.skill != null),
    "slot-active": !!slot.active,
    "slot-locked": !slot.available,
    "slot-cooldown": slot.cooldown > 0.01,
  };
}

// --- Analog joystick ---
const joystickState = reactive({});
function getJoy(id) {
  if (!joystickState[id]) {
    joystickState[id] = { active: false, pointerId: null, x: 0, y: 0, actions: new Set(), el: null };
  }
  return joystickState[id];
}
function setJoyEl(id, el) { getJoy(id).el = el || null; }

function joyKnobStyle(control) {
  const joy = getJoy(control.id);
  const size = Number(control.size || 140);
  const knob = Math.max(28, Math.round(size * 0.34));
  const tx = joy.active ? joy.x : 0;
  const ty = joy.active ? joy.y : 0;
  return {
    width: `${knob}px`,
    height: `${knob}px`,
    transform: `translate(calc(-50% + ${tx}px), calc(-50% + ${ty}px))`,
    opacity: joy.active ? "0.45" : "0",
  };
}

function setJoystickActions(id, nextActions) {
  const joy = getJoy(id);
  const next = new Set(nextActions);
  for (const a of MOVE_ACTIONS) {
    const had = joy.actions.has(a);
    const has = next.has(a);
    if (had && !has) actionUp(a);
    else if (!had && has) actionDown(a);
  }
  joy.actions = next;
}

function updateJoystickFromEvent(control, e) {
  const joy = getJoy(control.id);
  const el = joy.el || e.currentTarget;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  const dx = e.clientX - cx;
  const dy = e.clientY - cy;
  const size = Number(control.size || rect.width || 140);
  const radius = Math.max(20, size * 0.36);
  const dead = Math.max(8, size * 0.12);
  const dist = Math.hypot(dx, dy);
  const clamped = dist > radius ? radius / (dist || 1) : 1;
  joy.x = dx * clamped;
  joy.y = dy * clamped;

  const next = [];
  if (Math.abs(joy.x) > dead || Math.abs(joy.y) > dead) {
    if (joy.x < -dead) next.push(Action.MoveLeft);
    if (joy.x > dead) next.push(Action.MoveRight);
    if (joy.y < -dead) next.push(Action.MoveUp);
    if (joy.y > dead) next.push(Action.MoveDown);
  }
  setJoystickActions(control.id, next);
}

function joyDown(control, e) {
  if (e.pointerType === "mouse" && e.button !== 0) return;
  const joy = getJoy(control.id);
  joy.active = true;
  joy.pointerId = e.pointerId;
  activeClasses[control.id] = true;
  try { e.currentTarget?.setPointerCapture?.(e.pointerId); } catch {}
  updateJoystickFromEvent(control, e);
  e.preventDefault();
  e.stopPropagation();
}

function joyMove(control, e) {
  const joy = getJoy(control.id);
  if (!joy.active || joy.pointerId !== e.pointerId) return;
  updateJoystickFromEvent(control, e);
  e.preventDefault();
  e.stopPropagation();
}

function joyUp(control, e) {
  const joy = getJoy(control.id);
  if (!joy.active || (joy.pointerId != null && joy.pointerId !== e.pointerId)) return;
  setJoystickActions(control.id, []);
  joy.active = false;
  joy.pointerId = null;
  joy.x = 0;
  joy.y = 0;
  activeClasses[control.id] = false;
  e.preventDefault();
  e.stopPropagation();
}

function apiSetLayout(next) {
  if (!next || !Array.isArray(next.controls)) return;
  layout.value = next;
  saveLayout(layout.value);
}
function apiGetLayout() { return clone(layout.value); }
function apiResetLayout() { apiSetLayout(clone(DEFAULT_LAYOUT)); }
function apiUpdateControl(id, patch) {
  const next = apiGetLayout();
  const control = next.controls.find((c) => c && c.id === id);
  if (!control) return;
  Object.assign(control, patch || {});
  apiSetLayout(next);
}
function apiApply(state = props.state) {
  return mobileControlsVisible(state?.settings, state?.scene);
}

onMounted(() => {
  globalThis.__enzoMobileLayout = { get: apiGetLayout, set: apiSetLayout, reset: apiResetLayout, update: apiUpdateControl };
  globalThis.__enzoMobileControls = { render: () => {}, apply: apiApply };
});
onBeforeUnmount(() => {
  for (const id of Object.keys(joystickState)) {
    setJoystickActions(id, []);
  }
});
</script>

<template>
  <div
    id="dom-controls-root"
    class="dom-controls-root"
    :style="{ display: visible ? '' : 'none' }"
  >
    <template v-for="control in renderControls" :key="control.id || `${control.label}-${control.x}-${control.y}`">
      <div
        v-if="control.kind === 'joystick'"
        :ref="(el) => setJoyEl(control.id, el)"
        class="ui-joystick"
        :class="[anchorClass(control.anchor), { active: activeClasses[control.id] }]"
        :data-control-id="control.id || ''"
        :style="layoutVars(control)"
        @pointerdown="joyDown(control, $event)"
        @pointermove="joyMove(control, $event)"
        @pointerup="joyUp(control, $event)"
        @pointercancel="joyUp(control, $event)"
        @contextmenu.prevent
      >
        <div class="ui-joystick-base" aria-hidden="true"></div>
        <div class="ui-joystick-knob" :style="joyKnobStyle(control)" aria-hidden="true"></div>
      </div>

      <div
        v-else
        class="ui-btn"
        :class="[
          anchorClass(control.anchor),
          { active: activeClasses[control.id] },
          control.kind === 'slot' ? slotClasses(control) : '',
        ]"
        :data-control-id="control.id || ''"
        :data-kind="control.kind || null"
        :data-slot="control.slot != null ? String(control.slot) : null"
        :style="layoutVars(control)"
        role="button"
        tabindex="-1"
        @pointerdown="press(control, $event)"
        @pointerup="release(control, $event)"
        @pointercancel="release(control, $event)"
        @pointerleave="release(control, $event)"
        @contextmenu.prevent
      >
        <span v-if="control.kind === 'slot'" class="ui-label">
          <span class="ui-label__slot-num">{{ control.slot }}</span>
          <img
            v-if="slotInfo(control)?.skill_icon"
            class="ui-label__skill-icon"
            :src="slotInfo(control).skill_icon"
            :alt="slotInfo(control)?.skill_name || ''"
            :title="slotInfo(control)?.skill_name || ''"
          />
          <span v-else-if="slotSkillName(slotInfo(control))" class="ui-label__skill-name">{{ slotSkillName(slotInfo(control)) }}</span>
        </span>
        <span v-else class="ui-label">{{ control.label }}</span>

        <span
          v-if="control.kind === 'slot' && slotInfo(control)?.cooldown > 0.01"
          class="ui-slot-cd"
        >{{ slotInfo(control).cooldown.toFixed(1) }}</span>
      </div>
    </template>
  </div>
</template>
