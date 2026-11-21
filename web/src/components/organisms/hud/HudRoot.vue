<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { UiAction, closeMenuAndFocusGame, loadoutAssign, loadoutClear, sandboxApply, uiAction, unequipSlot } from "../../../bridge/bridge.js";
import { mobileControlsVisible } from "../../../lib/mobile.js";
import HudTopBar from "./HudTopBar.vue";
import HudActionBar from "./HudActionBar.vue";
import HudPhaseOverlay from "./HudPhaseOverlay.vue";
import HudToast from "./HudToast.vue";
import StatusPanel from "./panels/StatusPanel.vue";
import EquipmentPanel from "./panels/EquipmentPanel.vue";
import SkillsPanel from "./panels/SkillsPanel.vue";
import CustomizePanel from "./panels/CustomizePanel.vue";
import DebugPanel from "./panels/DebugPanel.vue";

const props = defineProps({
  state: { type: Object, default: null },
});

const openPanel = ref(null); // status | equipment | skills | customize | debug | null
const selectedSkillSlot = ref(0);
const fullscreen = ref(false);

const state = computed(() => props.state);
const showHud = computed(() => state.value?.scene === "game");
const player = computed(() => state.value?.player ?? null);
const labels = computed(() => state.value?.labels ?? {});
const phase = computed(() => state.value?.phase || "Playing");
const sandbox = computed(() => state.value?.sandbox ?? null);
const isSandbox = computed(() => !!sandbox.value?.active);

watch(showHud, (v) => { if (!v) openPanel.value = null; });

function isFullscreen() {
  return !!(document.fullscreenElement || document.webkitFullscreenElement);
}
function syncFullscreen() { fullscreen.value = isFullscreen(); }
async function toggleFullscreen() {
  try {
    if (isFullscreen()) {
      if (document.exitFullscreen) await document.exitFullscreen();
      else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
    } else {
      const root = document.documentElement;
      if (root.requestFullscreen) await root.requestFullscreen();
      else if (root.webkitRequestFullscreen) root.webkitRequestFullscreen();
    }
  } catch {}
  syncFullscreen();
}

function fullscreenLabel() {
  const lang = state.value?.settings?.language || "en";
  const active = fullscreen.value;
  if (lang === "pt") return active ? "Sair" : "Tela cheia";
  return active ? "Exit" : "Fullscreen";
}

function canvasCoordsFromClient(clientX, clientY) {
  const canvas = document.getElementById("glcanvas");
  if (!canvas) return null;
  const rect = canvas.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) return null;
  const x = (clientX - rect.left) / rect.width;
  const y = (clientY - rect.top) / rect.height;
  return { x, y };
}

function sandboxPlacementEnabled() {
  if (!isSandbox.value || openPanel.value !== "debug") return false;
  const tab = sandbox.value?.tab ?? 0;
  return tab === 1 || tab === 2 || tab === 3;
}

function onSandboxCanvasClick(e) {
  if (!sandboxPlacementEnabled()) return;
  const target = e.target;
  if (!(target && (target.id === "glcanvas" || target.tagName === "CANVAS"))) return;
  const coords = canvasCoordsFromClient(e.clientX, e.clientY);
  if (!coords) return;
  sandboxApply(coords.x, coords.y);
  e.preventDefault();
}

function onHudKeyDown(e) {
  if (e.key !== "Escape" || !openPanel.value) return;
  e.preventDefault();
  e.stopPropagation();
  closePanel();
}

onMounted(() => {
  syncFullscreen();
  document.addEventListener("fullscreenchange", syncFullscreen);
  document.addEventListener("webkitfullscreenchange", syncFullscreen);
  window.addEventListener("keydown", onHudKeyDown, true);
  window.addEventListener("pointerdown", onSandboxCanvasClick, true);
});
onBeforeUnmount(() => {
  document.removeEventListener("fullscreenchange", syncFullscreen);
  document.removeEventListener("webkitfullscreenchange", syncFullscreen);
  window.removeEventListener("keydown", onHudKeyDown, true);
  window.removeEventListener("pointerdown", onSandboxCanvasClick, true);
});

function act(code, a = 0, b = 0, c = 0) { uiAction(code, a, b, c); }

function closePanel() {
  closeMenuAndFocusGame(() => {
    openPanel.value = null;
  });
}

function togglePanel(panel) {
  if (openPanel.value === panel) closePanel();
  else openPanel.value = panel;
}

function closePanelOnOverlay(e) {
  if (openPanel.value === "debug" && isSandbox.value) return;
  if (e.target === e.currentTarget) closePanel();
}

const needsDistribution = computed(() => !!(player.value?.pending_level_up || (player.value?.points || 0) > 0));
const tabs = computed(() => {
  const base = [
    ["status", labels.value.status || "Status", needsDistribution.value],
    ["equipment", labels.value.equipment || "Equipment", false],
    ["skills", labels.value.skills || "Skills", false],
  ];
  if (isSandbox.value) {
    base.push(["debug", labels.value.sandbox_debug || "Debug", false]);
  }
  return base;
});

const attrDefs = computed(() => {
  const attrs = player.value?.attributes || {};
  return [
    ["Speed", attrs.speed ?? 0, 0],
    ["Strength", attrs.strength ?? 0, 1],
    ["Health", attrs.health ?? 0, 2],
    ["Aura", attrs.aura ?? 0, 3],
    ["Ego", attrs.ego ?? 0, 4],
  ];
});

const showActionBar = computed(() => showHud.value && player.value && !mobileControlsVisible(state.value?.settings, state.value?.scene));
const showPhaseOverlay = computed(() => showHud.value && (phase.value === "Dead" || phase.value === "Victory"));

const controls = computed(() => state.value?.controls || []);
function primaryKeyForAction(actionName) {
  const entry = controls.value.find((c) => c && c.action === actionName);
  if (entry && Array.isArray(entry.keys) && entry.keys.length > 0) return entry.keys[0];
  return "";
}
function slotKeyLabel(index) {
  return primaryKeyForAction(`Slot${index + 1}`) || String(index + 1);
}

function sandboxSelectBlock(blockCode, mode) {
  act(UiAction.SandboxSetBlockTool, mode, blockCode);
}
</script>

<template>
  <div id="dom-hud-root" class="dom-hud-root" :style="{ pointerEvents: 'none' }">
    <template v-if="showHud && player">
      <HudTopBar
        :player="player"
        :labels="labels"
        :tabs="tabs"
        :open-panel="openPanel"
        :fullscreen="fullscreen"
        :fullscreen-label="fullscreenLabel()"
        :menu-label="labels.menu || 'Menu'"
        :settings="state.settings || {}"
        @toggle-panel="togglePanel"
        @toggle-fullscreen="toggleFullscreen"
        @open-menu="act(UiAction.HudOpenMenu)"
      />

      <HudActionBar
        v-if="showActionBar"
        :slots="player.slots || []"
        :slot-key-label="slotKeyLabel"
        @open-skills="togglePanel('skills')"
      />

      <div
        v-if="openPanel"
        class="dom-game-panel-overlay"
        :class="{ 'is-debug-sidebar': openPanel === 'debug' && isSandbox }"
        @click="closePanelOnOverlay"
      >
        <div class="dom-game-panel" :class="{ 'debug-sidebar-panel': openPanel === 'debug' && isSandbox }" style="pointer-events: auto" @click.stop>
          <StatusPanel
            v-if="openPanel === 'status'"
            :player="player"
            :labels="labels"
            :attr-defs="attrDefs"
            @close="closePanel()"
            @allocate="(code) => act(UiAction.HudAllocate, code)"
          />

          <EquipmentPanel
            v-else-if="openPanel === 'equipment'"
            :player="player"
            :labels="labels"
            @close="closePanel()"
            @unequip="(slotCode) => unequipSlot(slotCode)"
          />

          <CustomizePanel
            v-else-if="openPanel === 'customize'"
            :state="state"
            :player="player"
            :labels="labels"
            @close="closePanel()"
          />

          <SkillsPanel
            v-else-if="openPanel === 'skills'"
            :state="state"
            :player="player"
            :labels="labels"
            :selected-skill-slot="selectedSkillSlot"
            :slot-key-label="slotKeyLabel"
            @close="closePanel()"
            @select-slot="(i) => (selectedSkillSlot = i)"
            @assign="(slot, skill) => loadoutAssign(slot, skill)"
            @clear="(slot) => loadoutClear(slot)"
          />

          <DebugPanel
            v-else-if="openPanel === 'debug' && isSandbox"
            :state="state"
            :player="player"
            :labels="labels"
            :sandbox="sandbox"
            @close="closePanel()"
            @set-tab="(tab) => act(UiAction.SandboxSetTab, tab)"
            @select-block="sandboxSelectBlock"
            @set-enemy="(code) => act(UiAction.SandboxSetEnemy, code)"
            @set-item="(code) => act(UiAction.SandboxSetItem, code)"
            @toggle-sigma="act(UiAction.SandboxToggleSigma)"
            @grant-levels="(n) => act(UiAction.SandboxGrantLevels, n)"
            @set-level="(n) => act(UiAction.SandboxSetLevel, n)"
            @reset-attrs="act(UiAction.SandboxResetAttrs)"
            @die="act(UiAction.SandboxDie)"
            @reset-sandbox="act(UiAction.SandboxReset)"
          />
        </div>
      </div>
    </template>

    <HudToast v-if="showHud" :message="state?.game_toast || ''" />

    <HudPhaseOverlay
      v-if="showPhaseOverlay"
      :phase="phase"
      :labels="labels"
      @restart="act(UiAction.HudRestart)"
      @open-menu="act(UiAction.HudOpenMenu)"
    />
  </div>
</template>
