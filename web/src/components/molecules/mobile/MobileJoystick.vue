<script setup>
defineProps({
  control: { type: Object, required: true },
  active: { type: Boolean, default: false },
  anchorClass: { type: String, default: "anchor-bottom-left" },
  layoutVars: { type: Object, default: () => ({}) },
  knobStyle: { type: Object, default: () => ({}) },
  setEl: { type: Function, default: null },
});
defineEmits(["pointerdown", "pointermove", "pointerup", "pointercancel", "contextmenu"]);
</script>

<template>
  <div
    :ref="(el) => setEl && setEl(el)"
    class="ui-joystick"
    :class="[anchorClass, { active }]"
    :data-control-id="control.id || ''"
    :style="layoutVars"
    @pointerdown="$emit('pointerdown', $event)"
    @pointermove="$emit('pointermove', $event)"
    @pointerup="$emit('pointerup', $event)"
    @pointercancel="$emit('pointercancel', $event)"
    @contextmenu.prevent="$emit('contextmenu', $event)"
  >
    <div class="ui-joystick-base" aria-hidden="true"></div>
    <div class="ui-joystick-knob" :style="knobStyle" aria-hidden="true"></div>
  </div>
</template>
