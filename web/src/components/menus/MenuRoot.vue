<script setup>
import { computed } from "vue";
import { UiAction, uiAction } from "../../bridge/bridge.js";

const props = defineProps({
  state: { type: Object, default: null },
});

const menu = computed(() => props.state?.menu ?? null);
const labels = computed(() => props.state?.labels ?? {});
const settings = computed(() => props.state?.settings ?? {});
const show = computed(() => !!(props.state && (props.state.scene === "menu" || props.state.scene === "menu_over_game") && menu.value));

function act(code, a = 0, b = 0, c = 0) {
  uiAction(code, a, b, c);
}

function setScale(delta) {
  const current = Number(settings.value.interface_scale ?? 1);
  const pct = Math.round(Math.min(200, Math.max(75, (current + delta) * 100)));
  act(UiAction.SetInterfaceScale, Math.floor(pct / 256), pct % 256);
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
        <template v-if="menu.screen === 'main' || !['saves', 'options'].includes(menu.screen)">
          <button v-if="menu.has_background_game" type="button" class="dom-btn primary" @click.prevent.stop="act(UiAction.MenuResume)">
            {{ labels.resume }}
          </button>
          <button type="button" class="dom-btn primary" @click.prevent.stop="act(UiAction.MenuOpenSaves)">
            {{ labels.play }}
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
