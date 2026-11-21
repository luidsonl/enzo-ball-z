<script setup>
import { computed, ref, watch, onBeforeUnmount } from "vue";
import { UiAction, uiAction, setPlayableCharacter, setVisualProfileField } from "../../../bridge/bridge.js";
import AppButton from "../../atoms/AppButton.vue";
import AppTitle from "../../atoms/AppTitle.vue";
import AppSubtitle from "../../atoms/AppSubtitle.vue";
import Keycap from "../../atoms/Keycap.vue";
import ColorSwatch from "../../atoms/ColorSwatch.vue";
import SaveSlotCard from "../../molecules/SaveSlotCard.vue";
import OptionRow from "../../molecules/OptionRow.vue";

const props = defineProps({
  state: { type: Object, default: null },
});

const showHelp = ref(false);
const showResetConfirm = ref(false);

const menu = computed(() => props.state?.menu ?? null);
const labels = computed(() => props.state?.labels ?? {});
const settings = computed(() => props.state?.settings ?? {});
const player = computed(() => props.state?.player ?? null);
const show = computed(() => !!(props.state && (props.state.scene === "menu" || props.state.scene === "menu_over_game") && menu.value));
const controls = computed(() => props.state?.controls ?? []);
const supportedKeys = computed(() => props.state?.supported_keys ?? []);
const isEnglish = computed(() => (settings.value?.language || "pt-BR") === "en");
const isMainScreen = computed(() => {
  const s = menu.value?.screen || "main";
  return s === "main" || !["saves", "options", "customize", "controls"].includes(s);
});
const sandboxUnlocked = computed(() => !!menu.value?.sandbox_unlocked);

const capturing = ref(false);
const capturingActionCode = ref(null);

function supportedIndexByName(name) {
  const found = supportedKeys.value.find((k) => k.name === name);
  return found ? found.index : null;
}

function stopCapture() {
  capturing.value = false;
  capturingActionCode.value = null;
}

function startCapture(actionCode) {
  capturing.value = true;
  capturingActionCode.value = actionCode;
}

function openResetConfirm() {
  showHelp.value = false;
  showResetConfirm.value = true;
}

function confirmResetAllData() {
  try {
    if (typeof window !== "undefined") {
      window.localStorage?.clear?.();
      window.sessionStorage?.clear?.();
    }
  } catch (_) {
    // ignore and still reload
  }
  showResetConfirm.value = false;
  if (typeof window !== "undefined") {
    window.location.reload();
  }
}

function normalizeCaptureCode(ev) {
  const code = ev.code || "";
  if (code && supportedIndexByName(code) != null) return code;
  const key = ev.key || "";
  const fallbacks = {
    " ": "Space",
    Esc: "Escape",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    Control: ev.location === 2 ? "ControlRight" : "ControlLeft",
    Shift: ev.location === 2 ? "ShiftRight" : "ShiftLeft",
    Alt: ev.location === 2 ? "AltRight" : "AltLeft",
    Meta: ev.location === 2 ? "MetaRight" : "MetaLeft",
  };
  if (fallbacks[key] && supportedIndexByName(fallbacks[key]) != null) return fallbacks[key];
  if (/^[a-z]$/i.test(key)) {
    const name = `Key${key.toUpperCase()}`;
    if (supportedIndexByName(name) != null) return name;
  }
  if (/^[0-9]$/.test(key)) {
    const name = `Digit${key}`;
    if (supportedIndexByName(name) != null) return name;
  }
  return code || key;
}

function onCaptureKey(ev) {
  if (!capturing.value || capturingActionCode.value == null) return;
  ev.preventDefault();
  ev.stopPropagation();
  if (typeof ev.stopImmediatePropagation === "function") ev.stopImmediatePropagation();

  if (ev.key === "Escape" || ev.code === "Escape") {
    stopCapture();
    return;
  }
  const name = normalizeCaptureCode(ev);
  const idx = supportedIndexByName(name);
  if (idx == null) return;
  uiAction(UiAction.ControlsSetKey, capturingActionCode.value, Number(idx), 0);
  stopCapture();
}

function onMenuUnlockKey(ev) {
  if (!show.value || capturing.value) return;
  if (ev.key === "6" || ev.code === "Digit6" || ev.code === "Numpad6") {
    act(UiAction.SandboxUnlockDigit, 6);
  } else if (ev.key === "7" || ev.code === "Digit7" || ev.code === "Numpad7") {
    act(UiAction.SandboxUnlockDigit, 7);
  }
}

window.addEventListener("keydown", onCaptureKey, true);
window.addEventListener("keydown", onMenuUnlockKey);
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onCaptureKey, true);
  window.removeEventListener("keydown", onMenuUnlockKey);
});

watch(show, (v) => {
  if (!v) {
    showHelp.value = false;
    showResetConfirm.value = false;
    stopCapture();
  }
});
watch(() => menu.value?.screen, (screen) => {
  showHelp.value = false;
  showResetConfirm.value = false;
  if (screen !== "controls") stopCapture();
});

function act(code, a = 0, b = 0, c = 0) {
  uiAction(code, a, b, c);
}

function setScale(delta) {
  if (settings.value?.interface_scale_auto) return;
  const current = Number(settings.value.interface_scale ?? 1);
  const pct = Math.round(Math.min(200, Math.max(75, (current + delta) * 100)));
  act(UiAction.SetInterfaceScale, Math.floor(pct / 256), pct % 256);
}

function toggleInterfaceScaleAuto() {
  const enabled = !settings.value?.interface_scale_auto;
  act(UiAction.SetInterfaceScaleAuto, enabled ? 1 : 0, 0);
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
    <div v-if="show" class="dom-menu-wrap" style="position: relative;">
      <AppButton
        v-if="isMainScreen"
        class="dom-help-btn"
        variant="secondary"
        size="small"
        :title="isEnglish ? 'Controls and instructions' : 'Comandos e instruções'"
        :aria-label="isEnglish ? 'Controls and instructions' : 'Comandos e instruções'"
        @click.prevent.stop="showHelp = true"
      >?</AppButton>

      <div class="dom-panel">
        <template v-if="menu.screen === 'main' || !['saves', 'options', 'customize', 'controls'].includes(menu.screen)">
          <AppButton v-if="menu.has_background_game" variant="primary" @click.prevent.stop="act(UiAction.MenuResume)">
            {{ labels.resume }}
          </AppButton>
          <AppButton variant="primary" @click.prevent.stop="act(UiAction.MenuOpenSaves)">
            {{ labels.play }}
          </AppButton>
          <AppButton v-if="sandboxUnlocked" variant="secondary" @click.prevent.stop="act(UiAction.MenuOpenSandbox)">
            {{ labels.sandbox || 'Sandbox' }}
          </AppButton>
          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuOpenCustomize)">
            {{ labels.character_editor || labels.customize || 'Character Editor' }}
          </AppButton>
          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuOpenOptions)">
            {{ labels.options }}
          </AppButton>
        </template>

        <template v-else-if="menu.screen === 'saves'">
          <AppTitle>{{ labels.saves_title }}</AppTitle>
          <AppSubtitle>{{ labels.choose_slot }}</AppSubtitle>
          <SaveSlotCard
            v-for="slot in (menu.slots || [])"
            :key="slot.slot"
            :slot-data="slot"
            :labels="labels"
            @load="act(UiAction.MenuStart, slot.slot, 0)"
            @delete="act(UiAction.MenuRequestDelete, slot.slot)"
            @new-game="act(UiAction.MenuStart, slot.slot, 1)"
          />
          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </AppButton>
        </template>

        <template v-else-if="menu.screen === 'options'">
          <AppTitle>{{ labels.options_title }}</AppTitle>

          <OptionRow :label="labels.language">
            <AppButton size="small" :selected="settings.language === 'pt-BR'" @click.prevent.stop="act(UiAction.SetLanguage, 0)">PT-BR</AppButton>
            <AppButton size="small" :selected="settings.language === 'en'" @click.prevent.stop="act(UiAction.SetLanguage, 1)">EN</AppButton>
          </OptionRow>

          <OptionRow :label="labels.mobile_controls">
            <AppButton size="small" :selected="settings.mobile_controls === 'on'" @click.prevent.stop="act(UiAction.SetMobileControls, 1)">{{ labels.on }}</AppButton>
            <AppButton size="small" :selected="settings.mobile_controls === 'off'" @click.prevent.stop="act(UiAction.SetMobileControls, 0)">{{ labels.off }}</AppButton>
            <AppButton size="small" :selected="settings.mobile_controls === 'auto'" @click.prevent.stop="act(UiAction.SetMobileControls, 2)">{{ labels.auto }}</AppButton>
          </OptionRow>

          <OptionRow :label="`${labels.interface_size}: ${Math.round((settings.interface_scale || 1) * 100)}%`">
            <label class="dom-btn small" :class="{ selected: !!settings.interface_scale_auto }" style="display:inline-flex; align-items:center; gap:6px; cursor:pointer;">
              <input
                type="checkbox"
                :checked="!!settings.interface_scale_auto"
                @change.prevent.stop="toggleInterfaceScaleAuto()"
                style="pointer-events:auto"
              />
              <span>{{ labels.auto || (isEnglish ? 'Auto' : 'Automático') }}</span>
            </label>
            <AppButton size="small" :disabled="!!settings.interface_scale_auto" @click.prevent.stop="setScale(-0.25)">-</AppButton>
            <AppButton size="small" :disabled="!!settings.interface_scale_auto" @click.prevent.stop="setScale(0.25)">+</AppButton>
          </OptionRow>

          <OptionRow :label="isEnglish ? 'Controls' : 'Controles'">
            <AppButton size="small" @click.prevent.stop="act(UiAction.MenuOpenControls)">
              {{ isEnglish ? 'Configure controls' : 'Configurar controles' }}
            </AppButton>
          </OptionRow>

          <OptionRow :label="isEnglish ? 'Reset all data' : 'Resetar todos os dados'" danger>
            <AppButton size="small" variant="danger" @click.prevent.stop="openResetConfirm">
              {{ isEnglish ? 'Reset all data' : 'Resetar todos os dados' }}
            </AppButton>
          </OptionRow>

          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </AppButton>
        </template>

        <template v-else-if="menu.screen === 'customize'">
          <AppTitle>{{ labels.character_editor || labels.customize || 'Character Editor' }}</AppTitle>

          <div v-if="player" class="dom-customize-editor">
            <OptionRow :label="labels.character || 'Character'">
              <AppButton size="small" :selected="(player.character_code ?? 0) === 0" @click.prevent.stop="setPlayableCharacter(0)">Enzo</AppButton>
              <AppButton size="small" :selected="(player.character_code ?? 0) === 1" @click.prevent.stop="setPlayableCharacter(1)">Valentina</AppButton>
            </OptionRow>

            <OptionRow :label="labels.skin_tone || (isEnglish ? 'Skin tone' : 'Tom de pele')" class="dom-swatch-option">
              <div class="dom-option-buttons dom-swatch-row">
                <ColorSwatch v-for="(c, i) in SKIN_COLORS" :key="'s'+i" :color="c" :selected="isSelectedColor('skin_tone', i)" :label="(labels.skin_tone||'Skin') + ' ' + (i+1)" @select="setProfileField(0, i)" />
              </div>
            </OptionRow>

            <OptionRow :label="labels.hair_color || (isEnglish ? 'Hair color' : 'Cor do cabelo')" class="dom-swatch-option">
              <div class="dom-option-buttons dom-swatch-row">
                <ColorSwatch v-for="(c, i) in HAIR_COLORS" :key="'h'+i" :color="c" :selected="isSelectedColor('hair_color', i)" :label="(labels.hair_color||'Hair color') + ' ' + (i+1)" @select="setProfileField(2, i)" />
              </div>
            </OptionRow>

            <OptionRow :label="`${labels.hair_style || (isEnglish ? 'Hair style' : 'Estilo de cabelo')}: ${(profileValue('hair_style') ?? 0) + 1}`">
              <AppButton size="small" @click.prevent.stop="cycle('hair_style', 1, 7, -1)">−</AppButton>
              <AppButton size="small" @click.prevent.stop="cycle('hair_style', 1, 7, 1)">+</AppButton>
            </OptionRow>

            <OptionRow :label="labels.outfit_top || (isEnglish ? 'Shirt color (predominant)' : 'Cor da camiseta (predominante)')" class="dom-swatch-option">
              <div class="dom-option-buttons dom-swatch-row">
                <ColorSwatch v-for="(c, i) in CLOTH_COLORS" :key="'ot'+i" :color="c" :selected="isSelectedColor('outfit_top', i)" :label="(labels.outfit_top||'Shirt') + ' ' + (i+1)" @select="setProfileField(4, i)" />
              </div>
            </OptionRow>

            <OptionRow :label="labels.outfit_bottom || (isEnglish ? 'Pants color (predominant)' : 'Cor da calça (predominante)')" class="dom-swatch-option">
              <div class="dom-option-buttons dom-swatch-row">
                <ColorSwatch v-for="(c, i) in CLOTH_COLORS" :key="'ob'+i" :color="c" :selected="isSelectedColor('outfit_bottom', i)" :label="(labels.outfit_bottom||'Pants') + ' ' + (i+1)" @select="setProfileField(5, i)" />
              </div>
            </OptionRow>

            <OptionRow :label="labels.shoes || (isEnglish ? 'Details color' : 'Cor dos detalhes')" class="dom-swatch-option">
              <div class="dom-option-buttons dom-swatch-row">
                <ColorSwatch v-for="(c, i) in CLOTH_COLORS" :key="'od'+i" :color="c" :selected="isSelectedColor('shoes', i)" :label="(labels.shoes||'Details') + ' ' + (i+1)" @select="setProfileField(6, i)" />
              </div>
            </OptionRow>

            <OptionRow :label="`${labels.face || (isEnglish ? 'Face' : 'Rosto')}: ${(profileValue('face') ?? 0) + 1}`">
              <AppButton size="small" @click.prevent.stop="cycle('face', 3, 7, -1)">−</AppButton>
              <AppButton size="small" @click.prevent.stop="cycle('face', 3, 7, 1)">+</AppButton>
            </OptionRow>
          </div>

          <p v-else class="dom-subtitle">{{ labels.customize_no_player || (isEnglish ? 'Character editor unavailable.' : 'Editor de personagem indisponível.') }}</p>

          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuBack)">
            {{ labels.back }}
          </AppButton>
        </template>

        <template v-else-if="menu.screen === 'controls'">
          <AppTitle>{{ isEnglish ? 'Controls' : 'Controles' }}</AppTitle>
          <p class="dom-subtitle" style="margin-bottom:10px">
            {{ isEnglish ? 'Press a key to add it to an action. A key can only be used for one action, but an action can have multiple keys. Changes are saved automatically.' : 'Pressione uma tecla para adicioná-la a uma ação. Uma tecla não pode ter mais de uma ação, mas uma ação pode ter mais de uma tecla. As alterações são salvas automaticamente.' }}
          </p>

          <div class="dom-controls-config-actions">
            <AppButton size="small" variant="secondary" @click.prevent.stop="act(UiAction.ControlsResetDefaults)">
              {{ isEnglish ? 'Reset defaults' : 'Restaurar padrão' }}
            </AppButton>
          </div>

          <div class="dom-controls-config-list">
            <div v-for="c in controls" :key="c.action" class="dom-control-config-row">
              <div class="dom-control-config-main">
                <div class="dom-control-label">{{ c.label }}</div>
                <div class="dom-control-keys">
                  <template v-if="c.keys && c.keys.length">
                    <Keycap v-for="(k, i) in c.keys" :key="c.action + '-' + i">{{ k }}</Keycap>
                  </template>
                  <Keycap v-else muted>—</Keycap>
                </div>
              </div>
              <div class="dom-control-config-edit">
                <AppButton
                  size="small"
                  :selected="capturing && capturingActionCode === c.action_code"
                  @click.prevent.stop="startCapture(c.action_code)"
                >
                  {{ capturing && capturingActionCode === c.action_code ? (isEnglish ? 'Press a key… (Esc to cancel)' : 'Pressione uma tecla… (Esc cancela)') : (isEnglish ? 'Press key to set' : 'Pressionar tecla') }}
                </AppButton>
                <AppButton size="small" variant="secondary" @click.prevent.stop="act(UiAction.ControlsClearAction, c.action_code, 0, 0)">
                  {{ isEnglish ? 'Clear' : 'Limpar' }}
                </AppButton>
              </div>
            </div>
          </div>

          <AppButton variant="secondary" @click.prevent.stop="stopCapture(); act(UiAction.MenuBack)" style="margin-top:12px">
            {{ labels.back }}
          </AppButton>
        </template>
      </div>
    </div>

    <div v-if="show && menu.pending_delete != null" class="dom-modal-overlay">
      <div class="dom-modal">
        <AppTitle tag="h3">{{ labels.slot }} {{ (menu.pending_delete ?? 0) + 1 }}</AppTitle>
        <p>{{ labels.delete_confirm }}</p>
        <div class="dom-modal-actions">
          <AppButton variant="secondary" @click.prevent.stop="act(UiAction.MenuCancelDelete)">{{ labels.cancel }}</AppButton>
          <AppButton variant="danger" @click.prevent.stop="act(UiAction.MenuConfirmDelete)">{{ labels.confirm }}</AppButton>
        </div>
      </div>
    </div>

    <div v-if="show && showResetConfirm" class="dom-modal-overlay" @click.self="showResetConfirm = false">
      <div class="dom-modal dom-reset-modal">
        <AppTitle tag="h3">{{ isEnglish ? 'Reset all data?' : 'Resetar todos os dados?' }}</AppTitle>
        <p class="dom-subtitle" style="margin: 8px 0 14px;">
          {{ isEnglish
            ? 'This will delete all save slots and all site data stored in this browser (settings, language, controls, and customization). This action cannot be undone.'
            : 'Isso vai apagar todos os slots de save e todos os dados do site armazenados neste navegador (configurações, idioma, controles e personalização). Essa ação não pode ser desfeita.' }}
        </p>
        <div class="dom-modal-actions">
          <AppButton variant="secondary" @click.prevent.stop="showResetConfirm = false">{{ labels.cancel || (isEnglish ? 'Cancel' : 'Cancelar') }}</AppButton>
          <AppButton variant="danger" @click.prevent.stop="confirmResetAllData">{{ isEnglish ? 'Reset everything' : 'Resetar tudo' }}</AppButton>
        </div>
      </div>
    </div>

    <div v-if="show && showHelp" class="dom-modal-overlay dom-help-overlay" @click.self="showHelp = false">
      <div class="dom-modal dom-help-modal">
        <div class="dom-help-header">
          <AppTitle tag="h3">{{ isEnglish ? 'Controls & Instructions' : 'Comandos e Instruções' }}</AppTitle>
          <AppButton size="small" variant="secondary" @click.prevent.stop="showHelp = false">{{ labels.close || (isEnglish ? 'Close' : 'Fechar') }}</AppButton>
        </div>

        <div class="dom-help-body">
          <section class="dom-help-section">
            <h4>{{ isEnglish ? 'How to play' : 'Como jogar' }}</h4>
            <ul class="dom-help-list">
              <template v-if="isEnglish">
                <li>Move, jump, and fight to reach the flag at the end of the stage.</li>
                <li>Farm aura by alternating the <strong>6</strong> and <strong>7</strong> keys (SixSeven).</li>
                <li>Action slots trigger equipped skills/abilities (jump, attacks, specials, etc.).</li>
                <li>Defeat enemies to gain XP and level up, then allocate attribute points via the in-game interface.</li>
              </template>
              <template v-else>
                <li>Mova-se, pule e lute para alcançar a bandeira no fim da fase.</li>
                <li>Farme aura alternando as teclas <strong>6</strong> e <strong>7</strong> (SixSeven).</li>
                <li>Os slots de ação ativam habilidades equipadas (pulo, ataques, especiais etc.).</li>
                <li>Derrote inimigos para ganhar XP e subir de nível, depois distribua pontos de atributo pela interface do jogo.</li>
              </template>
            </ul>
          </section>

          <section class="dom-help-section">
            <h4>{{ isEnglish ? 'Configured controls' : 'Controles configurados' }}</h4>
            <div class="dom-controls-grid">
              <div v-for="c in controls" :key="c.action" class="dom-control-row">
                <div class="dom-control-label">{{ c.label }}</div>
                <div class="dom-control-keys">
                  <template v-if="c.keys && c.keys.length">
                    <Keycap v-for="(k, i) in c.keys" :key="c.action + '-' + i">{{ k }}</Keycap>
                  </template>
                  <Keycap v-else muted>—</Keycap>
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>
