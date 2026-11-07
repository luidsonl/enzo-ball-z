import { UiAction, loadoutAssign, loadoutClear, uiAction } from "./bridge.js";
import { mobileControlsVisible } from "./mobile-controls.js";
import { subscribe } from "./state.js";

let lastHudKey = null;
let lastState = null;
let openPanel = null; // "status" | "equipment" | "skills" | "items" | null
let selectedSkillSlot = 0; // 0-3, target slot for assignment in skills panel

function isFullscreen() {
  return !!(document.fullscreenElement || document.webkitFullscreenElement);
}

function fullscreenLabel(language) {
  const active = isFullscreen();
  if (language === "pt") return active ? "Sair" : "Tela cheia";
  return active ? "Exit" : "Fullscreen";
}

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
  } catch {
    // Ignore fullscreen errors (gesture/policy restrictions).
  }
  updateFullscreenButtons();
}

function updateFullscreenButtons() {
  document.querySelectorAll(".dom-hud-fullscreen-btn").forEach((btn) => {
    const lang = btn.dataset.lang || "en";
    btn.textContent = fullscreenLabel(lang);
    btn.title = btn.textContent;
    btn.setAttribute("aria-pressed", isFullscreen() ? "true" : "false");
  });
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function btn(label, className, onClick) {
  const b = document.createElement("button");
  b.type = "button";
  b.className = className;
  b.textContent = label;
  b.style.pointerEvents = "auto";
  b.style.touchAction = "manipulation";
  const handler = (e) => {
    e.preventDefault();
    e.stopPropagation();
    onClick(e);
  };
  b.addEventListener("pointerdown", handler);
  b.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); });
  return b;
}

function ensureHudRoot() {
  let root = document.getElementById("dom-hud-root");
  if (root) return root;
  const domUi = document.getElementById("dom-ui");
  if (!domUi) return null;
  root = document.createElement("div");
  root.id = "dom-hud-root";
  root.className = "dom-hud-root";
  domUi.appendChild(root);
  return root;
}

function pct(current, max) {
  if (!max || max <= 0) return 0;
  return Math.max(0, Math.min(100, (current / max) * 100));
}

function invalidateHud() {
  lastHudKey = null;
  if (lastState) renderHud(lastState);
}

function togglePanel(panel) {
  openPanel = openPanel === panel ? null : panel;
  invalidateHud();
}

function hudKey(state) {
  if (!state || state.scene !== "game") return `hidden:${state?.scene ?? ""}:${openPanel ?? ""}`;
  const mobile = mobileControlsVisible(state?.settings, state?.scene) ? 1 : 0;
  if (!state.player) return `hidden:${state.scene}:${state.phase ?? ""}:${state.game_toast ?? ""}:${mobile}:${openPanel ?? ""}`;
  const p = state.player;
  const slots = (p.slots || []).map(s => `${s.slot}:${s.skill ?? ""}:${s.available?1:0}:${s.active?1:0}:${s.cooldown.toFixed(1)}`).join("|");
  const catalog = (p.catalog || []).map(c => `${c.skill}:${c.available?1:0}:${c.equipped_slot ?? ""}`).join("|");
  const equip = (p.equipment_slots || []).map(e => `${e.slot}:${e.item ?? ""}`).join("|");
  const inv = (p.inventory || []).length;
  // While a game panel is open, ignore transient combat/farming values to avoid
  // rebuilding the panel DOM on every small HP/aura/xp change (which breaks clicks).
  const transient = openPanel ? "panel" : [
    Math.ceil(p.health), Math.ceil(p.max_health), Math.ceil(p.aura), Math.ceil(p.max_aura),
    Math.floor(p.xp), slots
  ].join(",");
  return [state.scene, state.phase ?? "", state.game_toast ?? "", mobile, openPanel ?? "", selectedSkillSlot,
    transient, p.level, p.points, p.pending_level_up ? 1 : 0,
    catalog, equip, inv,
    p.attributes?.speed, p.attributes?.strength, p.attributes?.health, p.attributes?.aura].join("::");
}

function renderBars(player) {
  const bars = el("div", "dom-hud-bars");
  function bar(label, current, max, kind) {
    const row = el("div", "dom-hud-bar-row");
    row.appendChild(el("span", "dom-hud-bar-label", `${label} ${Math.ceil(current)}/${Math.ceil(max)}`));
    const track = el("div", `dom-hud-track ${kind}`);
    const fill = el("div", "dom-hud-fill");
    fill.style.width = pct(current, max) + "%";
    track.appendChild(fill);
    row.appendChild(track);
    return row;
  }
  bars.appendChild(bar("HP", player.health, player.max_health, "hp"));
  bars.appendChild(bar("Aura", player.aura, player.max_aura, "aura"));
  bars.appendChild(bar(`Lv ${player.level}`, player.xp, player.xp_to_next, "xp"));
  return bars;
}

function renderTopActions(state, labels) {
  const topActions = el("div", "dom-hud-actions");
  topActions.style.pointerEvents = "auto";

  const fullscreenBtn = btn(fullscreenLabel(state.settings?.language || "en"), "dom-hud-btn dom-hud-fullscreen-btn", () => { void toggleFullscreen(); });
  fullscreenBtn.dataset.lang = state.settings?.language || "en";
  fullscreenBtn.title = fullscreenBtn.textContent;
  fullscreenBtn.setAttribute("aria-pressed", isFullscreen() ? "true" : "false");
  topActions.appendChild(fullscreenBtn);

  if (state.scene === "game") {
    topActions.appendChild(btn(labels.menu || "Menu", "dom-hud-btn dom-hud-menu-btn", () => uiAction(UiAction.HudOpenMenu)));
  }
  return topActions;
}

function renderPanelTabs(player, labels) {
  const tabs = el("div", "dom-hud-tabs");
  const needsDistribution = !!(player?.pending_level_up || (player?.points || 0) > 0);
  const defs = [
    ["status", labels.status || "Status", needsDistribution],
    ["equipment", labels.equipment || "Equipment", false],
    ["skills", labels.skills || "Skills", false],
    ["items", labels.inventory || "Items", false],
  ];
  for (const [id, label, blink] of defs) {
    const b = btn(label, `dom-hud-tab${openPanel === id ? " active" : ""}${blink ? " blink" : ""}`, () => togglePanel(id));
    tabs.appendChild(b);
  }
  return tabs;
}

function renderActionBar(state, player) {
  if (mobileControlsVisible(state?.settings, state?.scene)) return null;
  const actionBar = el("div", "dom-action-bar");
  actionBar.style.pointerEvents = "auto";
  (player.slots || []).forEach((slot, index) => {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "dom-action-slot";
    if (slot.active) b.classList.add("active");
    if (!slot.available) b.classList.add("locked");
    if (slot.cooldown > 0.01) b.classList.add("cooldown");
    const num = el("span", "slot-num", String(index + 1));
    const skill = el("span", "slot-skill", slot.skill_name || (slot.skill != null ? `S${slot.skill}` : "—"));
    b.appendChild(num);
    b.appendChild(skill);
    if (slot.cooldown > 0.01) b.appendChild(el("span", "slot-cd", slot.cooldown.toFixed(1)));
    b.title = slot.lock_reason || "";
    b.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); togglePanel("skills"); });
    actionBar.appendChild(b);
  });
  return actionBar;
}

function panelShell(title, labels, bodyBuilder) {
  const overlay = el("div", "dom-game-panel-overlay");
  const panel = el("div", "dom-game-panel");
  panel.style.pointerEvents = "auto";
  panel.addEventListener("click", (e) => e.stopPropagation());
  const header = el("div", "dom-game-panel-header");
  header.appendChild(el("h3", "dom-title", title));
  header.appendChild(btn(labels.close || "Close", "dom-btn small secondary", () => { openPanel = null; invalidateHud(); }));
  panel.appendChild(header);
  const body = el("div", "dom-game-panel-body");
  bodyBuilder(body);
  panel.appendChild(body);
  overlay.appendChild(panel);
  overlay.addEventListener("pointerdown", (e) => {
    if (e.target === overlay) { e.preventDefault(); openPanel = null; invalidateHud(); }
  });
  return overlay;
}

function renderStatusPanel(player, labels) {
  return panelShell(labels.status_title || labels.status || "Status", labels, (body) => {
    const attrs = player.attributes || {};
    const rows = [
      [labels.level || "Level", `${player.level}`],
      ["XP", `${Math.floor(player.xp)} / ${Math.floor(player.xp_to_next)}`],
      [labels.points || "Points", `${player.points}`],
      ["HP", `${Math.ceil(player.health)} / ${Math.ceil(player.max_health)}`],
      ["Aura", `${Math.ceil(player.aura)} / ${Math.ceil(player.max_aura)}`],
    ];
    const grid = el("div", "dom-status-grid");
    for (const [k, v] of rows) {
      const row = el("div", "dom-status-row");
      row.appendChild(el("span", "dom-status-key", k));
      row.appendChild(el("span", "dom-status-val", v));
      grid.appendChild(row);
    }
    body.appendChild(grid);

    const attrDefs = [
      ["Speed", attrs.speed ?? 0, 0],
      ["Strength", attrs.strength ?? 0, 1],
      ["Health", attrs.health ?? 0, 2],
      ["Aura", attrs.aura ?? 0, 3],
    ];
    body.appendChild(el("h4", "dom-subtitle", labels.points || "Points"));

    const pending = !!(player.pending_level_up || (player.points || 0) > 0);
    if ((player.points || 0) > 0) {
      body.appendChild(el("p", "muted", `${labels.points || "Points"}: ${player.points}`));
      const alloc = el("div", "dom-attr-grid");
      for (const [name, value, code] of attrDefs) {
        alloc.appendChild(btn(`+ ${name} (${value})`, "dom-btn small", () => {
          uiAction(UiAction.HudAllocate, code);
        }));
      }
      body.appendChild(alloc);
    } else {
      const attrsGrid = el("div", "dom-status-grid");
      for (const [name, value] of attrDefs) {
        const row = el("div", "dom-status-row");
        row.appendChild(el("span", "dom-status-key", name));
        row.appendChild(el("span", "dom-status-val", String(value)));
        attrsGrid.appendChild(row);
      }
      body.appendChild(attrsGrid);
    }

    // Points are distributed via attribute buttons; core clears pending state when points reach 0.
  });
}

function renderEquipmentPanel(player, labels) {
  return panelShell(labels.equipment_title || labels.equipment || "Equipment", labels, (body) => {
    const list = el("div", "dom-list");
    for (const slot of (player.equipment_slots || [])) {
      const row = el("div", "dom-list-row");
      row.appendChild(el("span", "dom-list-key", slot.slot_label || slot.slot));
      row.appendChild(el("span", "dom-list-val", slot.item || labels.no_item || "—"));
      list.appendChild(row);
    }
    if (!(player.equipment_slots || []).length) body.appendChild(el("p", "muted", labels.no_item || ""));
    else body.appendChild(list);
  });
}

function renderItemsPanel(player, labels) {
  return panelShell(labels.inventory_title || labels.inventory || "Items", labels, (body) => {
    const items = player.inventory || [];
    if (!items.length) {
      body.appendChild(el("p", "muted", labels.inventory_empty || "Empty"));
      return;
    }
    const list = el("div", "dom-list");
    for (const item of items) {
      const row = el("div", "dom-list-row");
      row.appendChild(el("span", "dom-list-val", item.name || item.kind));
      list.appendChild(row);
    }
    body.appendChild(list);
  });
}

function renderSkillsPanel(player, labels) {
  return panelShell(labels.skills_title || labels.skills || "Skills", labels, (body) => {
    body.appendChild(el("p", "dom-subtitle", labels.assign || "Assign to slot"));

    const slotPicker = el("div", "dom-slot-picker");
    for (let i = 0; i < 4; i++) {
      const slotState = (player.slots || []).find(s => s.slot === i) || {};
      const name = slotState.skill_name || (slotState.skill != null ? `S${slotState.skill}` : "—");
      const b = btn(`${i + 1}: ${name}`, `dom-btn small${selectedSkillSlot === i ? " selected" : ""}`, () => {
        selectedSkillSlot = i;
        invalidateHud();
      });
      slotPicker.appendChild(b);
    }
    body.appendChild(slotPicker);

    const clearRow = el("div", "dom-option-row");
    clearRow.appendChild(btn(labels.clear || "Clear", "dom-btn small danger", () => {
      loadoutClear(selectedSkillSlot);
    }));
    body.appendChild(clearRow);

    const list = el("div", "dom-skill-list");
    const unlocked = (player.catalog || []).filter((entry) => entry.available);
    if (!unlocked.length) {
      body.appendChild(el("p", "muted", labels.locked || "Locked"));
    }
    for (const entry of unlocked) {
      const row = el("div", `dom-skill-row${entry.equipped_slot != null ? " equipped" : ""}`);
      const info = el("div", "dom-skill-info");
      info.appendChild(el("div", "dom-skill-name", entry.name || `Skill ${entry.skill}`));
      const metaParts = [entry.category || ""];
      if (entry.equipped_slot != null) metaParts.push(`${labels.equipped || "Slot"} ${entry.equipped_slot + 1}`);
      if (entry.aura_cost > 0) metaParts.push(`Aura ${entry.aura_cost}`);
      if (entry.cooldown_max > 0) metaParts.push(`CD ${entry.cooldown_max}s`);
      info.appendChild(el("div", "dom-skill-meta", metaParts.filter(Boolean).join(" · ")));
      row.appendChild(info);

      const actions = el("div", "dom-skill-actions");
      actions.appendChild(btn(labels.assign || "Assign", "dom-btn small primary", () => {
        loadoutAssign(selectedSkillSlot, entry.skill);
      }));
      row.appendChild(actions);
      list.appendChild(row);
    }
    body.appendChild(list);
  });
}

function renderOpenPanel(player, labels) {
  if (!openPanel || !player) return null;
  if (openPanel === "status") return renderStatusPanel(player, labels);
  if (openPanel === "equipment") return renderEquipmentPanel(player, labels);
  if (openPanel === "skills") return renderSkillsPanel(player, labels);
  if (openPanel === "items") return renderItemsPanel(player, labels);
  return null;
}

function renderPhaseOverlay(state, player, labels, phase) {
  // Level-up distribution is handled in the Status menu (no exclusive level-up screen).
  if (!(phase === "Dead" || phase === "Victory")) return null;
  const overlay = el("div", "dom-phase-overlay");
  const panel = el("div", "dom-phase-panel");
  const title = phase === "Dead" ? (labels.game_over || "Game Over")
    : (labels.victory || "Victory");
  panel.appendChild(el("h2", "dom-title", title));

  const actions = el("div", "dom-modal-actions");
  actions.appendChild(btn(labels.restart || "Restart", "dom-btn primary", () => uiAction(UiAction.HudRestart)));
  actions.appendChild(btn(labels.menu || "Menu", "dom-btn secondary", () => uiAction(UiAction.HudOpenMenu)));
  panel.appendChild(actions);
  overlay.appendChild(panel);
  return overlay;
}

function renderHud(state) {
  lastState = state;
  const root = ensureHudRoot();
  if (!root) return;

  const key = hudKey(state);
  if (key === lastHudKey) return;
  lastHudKey = key;

  const showHud = state && state.scene === "game";
  if (!showHud) {
    openPanel = null;
    root.replaceChildren();
    root.style.pointerEvents = "none";
    return;
  }

  const player = state.player;
  const labels = state.labels || {};
  const phase = state.phase || "Playing";
  root.style.pointerEvents = "none";

  const frag = document.createDocumentFragment();

  if (player) {
    const top = el("div", "dom-hud-top");
    top.appendChild(renderBars(player));
    top.appendChild(renderPanelTabs(player, labels));
    top.appendChild(renderTopActions(state, labels));
    frag.appendChild(top);

    const actionBar = renderActionBar(state, player);
    if (actionBar) frag.appendChild(actionBar);

    const panel = renderOpenPanel(player, labels);
    if (panel) frag.appendChild(panel);
  }

  if (state.game_toast) {
    frag.appendChild(el("div", "dom-game-toast", state.game_toast));
  }

  const phaseOverlay = renderPhaseOverlay(state, player, labels, phase);
  if (phaseOverlay) frag.appendChild(phaseOverlay);

  root.replaceChildren(frag);
}

export function initHud() {
  document.addEventListener("fullscreenchange", updateFullscreenButtons);
  document.addEventListener("webkitfullscreenchange", updateFullscreenButtons);
  subscribe(renderHud);
}
