<script setup>
import { computed } from "vue";
import { UiAction, uiAction, setPlayableCharacter, setVisualProfileField } from "../../bridge/bridge.js";

const props = defineProps({
  state: { type: Object, default: null },
});

const menu = computed(() => props.state?.menu ?? null);
const labels = computed(() => props.state?.labels ?? {});
const settings = computed(() => props.state?.settings ?? {});
const player = computed(() => props.state?.player ?? null);
const show = computed(() => !!(props.state && (props.state.scene === "menu" || props.state.scene === "menu_over_game") && menu.value));

function act(code, a = 0, b = 0, c = 0) {
  uiAction(code, a, b, c);
}

function setScale(delta) {
  const current = Number(settings.value.interface_scale ?? 1);
  const pct = Math.round(Math.min(200, Math.max(75, (current + delta) * 100)));
  act(UiAction.SetInterfaceScale, Math.floor(pct / 256), pct % 256);
}

const SKIN_COLORS = ["#f6d7b0", "#efc89a", "#ddb07a", "#c89262", "#a8744c", "#86583a", "#f0e0c8", "#f2b8a8"];
const HAIR_COLORS = ["#5a3a22", "#1f1a1a", "#5c3d29", "#d1ad4d", "#9e3830", "#c7ccd9", "#4d85e0", "#b861d1"];
const CLOTH_COLORS = ["#4f7cff", "#4f84ff", "#f274a8", "#4dc773", "#f2b847", "#e6594d", "#8c73d9", "#c7ccd9"];

function profileValue(key) {
  return player.value?.visual_profile?.[key] ?? 0;
}
function setProfileField(field, value) {
  setVisualProfileField(field, value);
}
function cycle(key, field, max, dir) {
  const count = max + 1;
  const next = (profileValue(key) + dir + count) % count;
  setVisualProfileField(field, next);
}
function setMainClothingColor(value) {
  // "Main clothing color" applies to outfit top and bottom for a cohesive look.
  setVisualProfileField(4, value);
  setVisualProfileField(5, value);
}
function isSelectedColor(fieldKey, value) {
  return profileValue(fieldKey) === value;
}
</script>

<template>
  <div
    id="dom-menu-root"
    class="dom-menu-root"
    :class="{ active: show }"
    :style="{ pointerEvents: show ? 'auto' : 'none' }"
  >
    <div v-if="show" class="dom-menu-wrap">
      <div class="dom-panel">
        <template v-if="menu.screen === 'main' || !['saves', 'options', 'customize'].includes(menu.screen)">
          <button v-if="menu.has_background_game" type="button" class="dom-btn primary" @click.prevent.stop="act(UiAction.MenuResume)">
            {{ labels.resume }}
          </button>
          <button type="button" class="dom-btn primary" @click.prevent.stop="act(UiAction.MenuOpenSaves)">
            {{ labels.play }}
          </button>
          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuOpenCustomize)">
            {{ labels.character_editor || labels.customize || 'Character Editor' }}
          </button>
          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuOpenOptions)">
            {{ labels.options }}
          </button>
        </template>

        <template v-else-if="menu.screen === 'saves'">
          <h2 class="dom-title">{{ labels.saves_title }}</h2>
          <p class="dom-subtitle">{{ labels.choose_slot }}</p>
          <div v-for="slot in (menu.slots || [])" :key="slot.slot" class="dom-slot-card">
            <div class="dom-slot-title">{{ labels.slot }} {{ slot.slot + 1 }}</div>
            <template v-if="slot.occupied">
              <div class="dom-slot-meta">
                {{ labels.level }} {{ slot.level }} · {{ Math.floor(slot.xp) }}/{{ Math.floor(slot.xp_to_next) }} XP · {{ slot.points }} {{ labels.points }}
              </div>
              <div class="dom-slot-actions">
                <button type="button" class="dom-btn small primary" @click.prevent.stop="act(UiAction.MenuStart, slot.slot, 0)">
                  {{ labels.load }}
                </button>
                <button type="button" class="dom-btn small danger" @click.prevent.stop="act(UiAction.MenuRequestDelete, slot.slot)">
                  {{ labels.delete }}
                </button>
              </div>
            </template>
            <template v-else>
              <div class="dom-slot-meta muted">{{ labels.empty_slot }}</div>
              <div class="dom-slot-actions">
                <button type="button" class="dom-btn small primary" @click.prevent.stop="act(UiAction.MenuStart, slot.slot, 1)">
                  {{ labels.new_game }}
                </button>
              </div>
            </template>
          </div>
          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </button>
        </template>

        <template v-else-if="menu.screen === 'options'">
          <h2 class="dom-title">{{ labels.options_title }}</h2>

          <div class="dom-option-row">
            <div class="dom-option-label">{{ labels.language }}</div>
            <div class="dom-option-buttons">
              <button type="button" class="dom-btn small" :class="{ selected: settings.language === 'pt-BR' }" @click.prevent.stop="act(UiAction.SetLanguage, 0)">PT-BR</button>
              <button type="button" class="dom-btn small" :class="{ selected: settings.language === 'en' }" @click.prevent.stop="act(UiAction.SetLanguage, 1)">EN</button>
            </div>
          </div>

          <div class="dom-option-row">
            <div class="dom-option-label">{{ labels.mobile_controls }}</div>
            <div class="dom-option-buttons">
              <button type="button" class="dom-btn small" :class="{ selected: settings.mobile_controls === 'on' }" @click.prevent.stop="act(UiAction.SetMobileControls, 1)">{{ labels.on }}</button>
              <button type="button" class="dom-btn small" :class="{ selected: settings.mobile_controls === 'off' }" @click.prevent.stop="act(UiAction.SetMobileControls, 0)">{{ labels.off }}</button>
              <button type="button" class="dom-btn small" :class="{ selected: settings.mobile_controls === 'auto' }" @click.prevent.stop="act(UiAction.SetMobileControls, 2)">{{ labels.auto }}</button>
            </div>
          </div>

          <div class="dom-option-row">
            <div class="dom-option-label">{{ labels.interface_size }}: {{ Math.round((settings.interface_scale || 1) * 100) }}%</div>
            <div class="dom-option-buttons">
              <button type="button" class="dom-btn small" @click.prevent.stop="setScale(-0.25)">-</button>
              <button type="button" class="dom-btn small" @click.prevent.stop="setScale(0.25)">+</button>
            </div>
          </div>

          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </button>
        </template>

        <template v-else-if="menu.screen === 'customize'">
          <h2 class="dom-title">{{ labels.character_editor || labels.customize || 'Character Editor' }}</h2>
          <p class="dom-subtitle" style="margin-bottom:10px">
            {{ labels.customize_hint || 'Changes apply to all animation states (idle, walk, jump, attack, swim, etc.).' }}
          </p>

          <div v-if="player" class="dom-customize-editor">
            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.character || 'Character' }}</div>
              <div class="dom-option-buttons">
                <button type="button" class="dom-btn small" :class="{ selected: (player.character_code ?? 0) === 0 }" @click.prevent.stop="setPlayableCharacter(0)">Enzo</button>
                <button type="button" class="dom-btn small" :class="{ selected: (player.character_code ?? 0) === 1 }" @click.prevent.stop="setPlayableCharacter(1)">Valentina</button>
              </div>
            </div>

            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.skin_tone || 'Skin tone' }}</div>
              <div class="dom-option-buttons dom-swatch-row">
                <button v-for="(c, i) in SKIN_COLORS" :key="'s'+i" type="button" class="dom-swatch" :class="{ selected: isSelectedColor('skin_tone', i) }" :style="{ background: c }" :aria-label="(labels.skin_tone||'Skin') + ' ' + (i+1)" @click.prevent.stop="setProfileField(0, i)" />
              </div>
            </div>

            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.hair_color || 'Hair color' }}</div>
              <div class="dom-option-buttons dom-swatch-row">
                <button v-for="(c, i) in HAIR_COLORS" :key="'h'+i" type="button" class="dom-swatch" :class="{ selected: isSelectedColor('hair_color', i) }" :style="{ background: c }" :aria-label="(labels.hair_color||'Hair color') + ' ' + (i+1)" @click.prevent.stop="setProfileField(2, i)" />
              </div>
            </div>

            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.hair_style || 'Hair style' }}: {{ (profileValue('hair_style') ?? 0) + 1 }}</div>
              <div class="dom-option-buttons">
                <button type="button" class="dom-btn small" @click.prevent.stop="cycle('hair_style', 1, 7, -1)">−</button>
                <button type="button" class="dom-btn small" @click.prevent.stop="cycle('hair_style', 1, 7, 1)">+</button>
              </div>
            </div>

            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.main_clothing_color || labels.outfit_top || 'Main clothing color' }}</div>
              <div class="dom-option-buttons dom-swatch-row">
                <button v-for="(c, i) in CLOTH_COLORS" :key="'c'+i" type="button" class="dom-swatch" :class="{ selected: isSelectedColor('outfit_top', i) }" :style="{ background: c }" :aria-label="(labels.main_clothing_color||'Clothing') + ' ' + (i+1)" @click.prevent.stop="setMainClothingColor(i)" />
              </div>
            </div>

            <div class="dom-option-row">
              <div class="dom-option-label">{{ labels.face || 'Face' }}: {{ (profileValue('face') ?? 0) + 1 }}</div>
              <div class="dom-option-buttons">
                <button type="button" class="dom-btn small" @click.prevent.stop="cycle('face', 3, 7, -1)">−</button>
                <button type="button" class="dom-btn small" @click.prevent.stop="cycle('face', 3, 7, 1)">+</button>
              </div>
            </div>
          </div>

          <p v-else class="dom-subtitle">{{ labels.customize_no_player || 'Character editor unavailable.' }}</p>

          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </button>
        </template>
      </div>
    </div>

    <div v-if="show && menu.pending_delete != null" class="dom-modal-overlay">
      <div class="dom-modal">
        <h3 class="dom-title">{{ labels.slot }} {{ (menu.pending_delete ?? 0) + 1 }}</h3>
        <p>{{ labels.delete_confirm }}</p>
        <div class="dom-modal-actions">
          <button type="button" class="dom-btn secondary" @click.prevent.stop="act(UiAction.MenuCancelDelete)">{{ labels.cancel }}</button>
          <button type="button" class="dom-btn danger" @click.prevent.stop="act(UiAction.MenuConfirmDelete)">{{ labels.confirm }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
