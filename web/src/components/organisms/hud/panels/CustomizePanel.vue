<script setup>
import { computed } from "vue";
import PanelHeader from "../../../molecules/PanelHeader.vue";
import AppButton from "../../../atoms/AppButton.vue";
import { setPlayableCharacter, setVisualProfileField } from "../../../../bridge/bridge.js";

const props = defineProps({
  state: { type: Object, default: null },
  player: { type: Object, required: true },
  labels: { type: Object, default: () => ({}) },
});
defineEmits(["close"]);

const SKIN_COLORS = ["#f6d7b0", "#efc89a", "#ddb07a", "#c89262", "#a8744c", "#86583a", "#f0e0c8", "#f2b8a8"];
const HAIR_COLORS = ["#5a3a22", "#1f1a1a", "#5c3d29", "#d1ad4d", "#9e3830", "#c7ccd9", "#4d85e0", "#b861d1"];
const CLOTH_COLORS = ["#4f7cff", "#4f84ff", "#f274a8", "#4dc773", "#f2b847", "#e6594d", "#8c73d9", "#c7ccd9"];

const CUSTOM_SECTIONS = [
  { titleKey: "character", fields: [] },
  { titleKey: "skin_tone", fields: [{ code: 0, key: "skin_tone", max: 7, kind: "color", palette: SKIN_COLORS }] },
  { titleKey: "hair", fields: [
    { code: 1, key: "hair_style", labelKey: "hair_style", max: 7, kind: "style" },
    { code: 2, key: "hair_color", labelKey: "hair_color", max: 7, kind: "color", palette: HAIR_COLORS },
  ]},
  { titleKey: "face", fields: [{ code: 3, key: "face", max: 7, kind: "style" }] },
  { titleKey: "clothing_colors", fields: [
    { code: 4, key: "outfit_top", labelKey: "outfit_top", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 7, key: "outfit_top_detail1", labelKey: "outfit_top_detail1", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 8, key: "outfit_top_detail2", labelKey: "outfit_top_detail2", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 5, key: "outfit_bottom", labelKey: "outfit_bottom", max: 7, kind: "color", palette: CLOTH_COLORS },
    { code: 6, key: "shoes", labelKey: "shoes", max: 7, kind: "color", palette: CLOTH_COLORS },
  ]},
];

const isEnglish = computed(() => (props.state?.settings?.language || "pt-BR") === "en");

function profileValue(key) {
  return props.player?.visual_profile?.[key] ?? 0;
}
function setProfile(code, value) { setVisualProfileField(code, value); }
function cycleProfile(field, dir) {
  const max = (field.max ?? 7) + 1;
  const next = (profileValue(field.key) + dir + max) % max;
  setVisualProfileField(field.code, next);
}
function fieldLabel(field, fallbackKey) {
  const k = field.labelKey || fallbackKey || field.key;
  const fallbacks = {
    outfit_top: isEnglish.value ? "Shirt color (predominant)" : "Cor da camiseta (predominante)",
    outfit_bottom: isEnglish.value ? "Pants color (predominant)" : "Cor da calça (predominante)",
    shoes: isEnglish.value ? "Details color" : "Cor dos detalhes",
  };
  return props.labels[k] || fallbacks[k] || k.replace(/_/g, " ");
}
function sectionTitle(section) {
  const map = {
    hair: props.labels.hair_style || (isEnglish.value ? "Hair" : "Cabelo"),
    clothing_colors: isEnglish.value ? "Clothing colors (top, pants, details)" : "Cores das roupas (cima, calça, detalhes)",
  };
  return props.labels[section.titleKey] || map[section.titleKey] || section.titleKey.replace(/_/g, " ");
}
function swatchColor(field) {
  if (!field.palette) return null;
  return field.palette[profileValue(field.key) % field.palette.length];
}
</script>

<template>
  <PanelHeader
    :title="labels.customize || 'Customize Character'"
    :close-label="labels.close || 'Close'"
    @close="$emit('close')"
  />
  <div class="dom-game-panel-body">
    <p class="muted" style="margin:0 0 8px 0">
      {{ isEnglish
        ? 'Clothing uses 3 colors: shirt (predominant), pants (predominant), and details.'
        : 'As roupas usam 3 cores: cor da camiseta (predominante), calça (predominante) e detalhes.' }}
    </p>
    <div class="dom-list">
      <div class="dom-list-row" style="align-items:center; gap:10px; flex-wrap:wrap;">
        <span class="dom-list-key">{{ labels.character || 'Character' }}</span>
        <div style="display:flex; gap:8px; pointer-events:auto">
          <AppButton size="small" :variant="player.character_code === 0 ? 'primary' : 'secondary'" @click.prevent.stop="setPlayableCharacter(0)">Enzo</AppButton>
          <AppButton size="small" :variant="player.character_code === 1 ? 'primary' : 'secondary'" @click.prevent.stop="setPlayableCharacter(1)">Valentina</AppButton>
        </div>
        <span class="dom-list-val" style="margin-left:auto">{{ player.character || 'Enzo' }}</span>
      </div>

      <template v-for="section in CUSTOM_SECTIONS" :key="section.titleKey">
        <template v-if="section.fields.length">
          <div class="dom-list-row" style="margin-top:8px; opacity:0.9; font-weight:600; font-size:12px; text-transform:uppercase; letter-spacing:.04em">{{ sectionTitle(section) }}</div>
          <div v-for="field in section.fields" :key="field.code" class="dom-list-row" style="align-items:center; gap:8px;">
            <span class="dom-list-key">{{ fieldLabel(field, section.titleKey) }}</span>
            <span v-if="swatchColor(field)" :style="{ width:'18px', height:'18px', borderRadius:'50%', background: swatchColor(field), border:'2px solid rgba(255,255,255,.25)', flex:'0 0 auto' }"></span>
            <span class="dom-list-val">{{ profileValue(field.key) }}</span>
            <div style="margin-left:auto; display:flex; gap:6px; pointer-events:auto">
              <AppButton size="small" variant="secondary" @click.prevent.stop="cycleProfile(field, -1)">−</AppButton>
              <AppButton size="small" variant="secondary" @click.prevent.stop="cycleProfile(field, 1)">+</AppButton>
            </div>
          </div>
          <div v-if="section.fields.some(f => f.palette)" style="display:flex; gap:6px; flex-wrap:wrap; padding:4px 0 2px 0; pointer-events:auto">
            <template v-for="field in section.fields.filter(f => f.palette)" :key="'p'+field.code">
              <button v-for="(c, i) in field.palette" :key="field.code+':'+i" type="button"
                @click.prevent.stop="setProfile(field.code, i)"
                :title="fieldLabel(field, section.titleKey) + ' ' + i"
                :style="{ width:'22px', height:'22px', borderRadius:'6px', background:c, border: profileValue(field.key)===i ? '2px solid #fff' : '2px solid rgba(255,255,255,.15)', cursor:'pointer', padding:0 }"></button>
            </template>
          </div>
        </template>
      </template>
    </div>
    <p class="muted" style="margin-top:10px">{{ labels.customize_hint || 'Choose character (Enzo/Valentina), hair, face, clothing, and skin tone. Equipped items render as overlays.' }}</p>
  </div>
</template>
