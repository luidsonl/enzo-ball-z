<script setup>
defineProps({
  control: { type: Object, required: true },
  active: { type: Boolean, default: false },
  anchorClass: { type: String, default: "anchor-bottom-left" },
  layoutVars: { type: Object, default: () => ({}) },
  extraClasses: { type: [Object, Array, String], default: () => ({}) },
  slotInfo: { type: Object, default: null },
  slotSkillName: { type: String, default: "" },
});
defineEmits(["pointerdown", "pointerup", "pointercancel", "pointerleave", "contextmenu"]);
</script>

<template>
  <div
    class="ui-btn"
    :class="[anchorClass, { active }, extraClasses]"
    :data-control-id="control.id || ''"
    :data-kind="control.kind || null"
    :data-slot="control.slot != null ? String(control.slot) : null"
    :style="layoutVars"
    role="button"
    tabindex="-1"
    @pointerdown="$emit('pointerdown', $event)"
    @pointerup="$emit('pointerup', $event)"
    @pointercancel="$emit('pointercancel', $event)"
    @pointerleave="$emit('pointerleave', $event)"
    @contextmenu.prevent="$emit('contextmenu', $event)"
  >
    <span v-if="control.kind === 'slot'" class="ui-label">
      <span class="ui-label__slot-num">{{ control.slot }}</span>
      <img
        v-if="slotInfo?.skill_icon"
        class="ui-label__skill-icon"
        :src="slotInfo.skill_icon"
        :alt="slotInfo?.skill_name || ''"
        :title="slotInfo?.skill_name || ''"
      />
      <span v-else-if="slotSkillName" class="ui-label__skill-name">{{ slotSkillName }}</span>
    </span>
    <span v-else class="ui-label">{{ control.label }}</span>

    <span v-if="control.kind === 'slot' && slotInfo?.cooldown > 0.01" class="ui-slot-cd">
      {{ slotInfo.cooldown.toFixed(1) }}
    </span>
  </div>
</template>
