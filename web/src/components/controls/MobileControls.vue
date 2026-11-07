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
    { id: "left",  label: "←", actions: [Action.MoveLeft],  x: 16,  y: 80,  size: 64, anchor: "bottom-left" },
    { id: "down",  label: "↓", actions: [Action.MoveDown],  x: 88,  y: 16,  size: 64, anchor: "bottom-left" },
    { id: "right", label: "→", actions: [Action.MoveRight], x: 160, y: 80,  size: 64, anchor: "bottom-left" },
    { id: "up",    label: "↑", actions: [Action.Slot1],     x: 88,  y: 144, size: 64, anchor: "bottom-left" },
    { id: "six",   label: "6", actions: [Action.Six],       x: 16,  y: 224, size: 48, anchor: "bottom-left" },
    { id: "seven", label: "7", actions: [Action.Seven],     x: 72,  y: 224, size: 48, anchor: "bottom-left" },
    { id: "slot1", label: "1", actions: [Action.Slot1],     x: 90, y: 96,  size: 60, anchor: "bottom-right", kind: "slot", slot: 1 },
    { id: "slot2", label: "2", actions: [Action.Slot2],     x: 18,  y: 96,  size: 60, anchor: "bottom-right", kind: "slot", slot: 2 },
    { id: "slot3", label: "3", actions: [Action.Slot3],     x: 90, y: 24,  size: 60, anchor: "bottom-right", kind: "slot", slot: 3 },
    { id: "slot4", label: "4", actions: [Action.Slot4],     x: 18,  y: 24,  size: 60, anchor: "bottom-right", kind: "slot", slot: 4 },
  ],
};

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

// Only layout data (position/size) comes from JS; visual styling is in CSS.
function layoutVars(control) {
  return {
    "--control-size": `${Number(control.size || 56)}px`,
    "--control-x": `${Number(control.x || 0)}px`,
    "--control-y": `${Number(control.y || 0)}px`,
  };
}

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
onBeforeUnmount(() => {});
</script>

<template>
  <div
    id="dom-controls-root"
    class="dom-controls-root"
    :style="{ display: visible ? '' : 'none' }"
  >
    <button
      v-for="control in (layout.controls || []).filter(c => c && !c.hidden)"
      :key="control.id || `${control.label}-${control.x}-${control.y}`"
      type="button"
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
      tabindex="-1"
      @pointerdown="press(control, $event)"
      @pointerup="release(control, $event)"
      @pointercancel="release(control, $event)"
      @pointerleave="release(control, $event)"
      @contextmenu.prevent
    >
      <span v-if="control.kind === 'slot'" class="ui-label">
        <span class="ui-label__slot-num">{{ control.slot }}</span>
        <span v-if="slotSkillName(slotInfo(control))" class="ui-label__skill-name">{{ slotSkillName(slotInfo(control)) }}</span>
      </span>
      <span v-else class="ui-label">{{ control.label }}</span>

      <span
        v-if="control.kind === 'slot' && slotInfo(control)?.cooldown > 0.01"
        class="ui-slot-cd"
      >{{ slotInfo(control).cooldown.toFixed(1) }}</span>
    </button>
  </div>
</template>
