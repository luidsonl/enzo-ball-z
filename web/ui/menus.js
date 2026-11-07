import { UiAction, uiAction } from "./bridge.js";
import { subscribe } from "./state.js";

let lastMenuKey = null;

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function button(label, className, onClick) {
  const b = el("button", `dom-btn ${className || ""}`.trim(), label);
  b.type = "button";
  b.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    onClick();
  });
  return b;
}

function ensureMenuRoot() {
  let root = document.getElementById("dom-menu-root");
  if (root) return root;
  const domUi = document.getElementById("dom-ui");
  if (!domUi) return null;
  root = document.createElement("div");
  root.id = "dom-menu-root";
  root.className = "dom-menu-root";
  domUi.appendChild(root);
  return root;
}

function menuKey(state) {
  if (!state || (state.scene !== "menu" && state.scene !== "menu_over_game") || !state.menu) return "hidden";
  const m = state.menu;
  const slots = (m.slots || []).map(s => `${s.slot}:${s.occupied ? 1 : 0}:${s.level ?? ""}:${s.xp ?? ""}:${s.points ?? ""}`).join("|");
  return [
    state.scene, m.screen, m.has_background_game ? 1 : 0, m.pending_delete ?? "", m.toast ?? "",
    slots, state.settings?.language ?? "", state.settings?.mobile_controls ?? "", state.settings?.interface_scale ?? "",
    state.labels?.play ?? "", state.labels?.options ?? ""
  ].join("::");
}

function renderMain(panel, menu, labels) {
  if (menu.has_background_game) {
    panel.appendChild(button(labels.resume, "primary", () => uiAction(UiAction.MenuResume)));
  }
  panel.appendChild(button(labels.play, "primary", () => uiAction(UiAction.MenuOpenSaves)));
  panel.appendChild(button(labels.options, "secondary", () => uiAction(UiAction.MenuOpenOptions)));
}

function renderSaves(panel, menu, labels) {
  panel.appendChild(el("h2", "dom-title", labels.saves_title));
  panel.appendChild(el("p", "dom-subtitle", labels.choose_slot));
  for (const slot of (menu.slots || [])) {
    const card = el("div", "dom-slot-card");
    const title = el("div", "dom-slot-title", `${labels.slot} ${slot.slot + 1}`);
    card.appendChild(title);
    if (slot.occupied) {
      card.appendChild(el("div", "dom-slot-meta",
        `${labels.level} ${slot.level} · ${Math.floor(slot.xp)}/${Math.floor(slot.xp_to_next)} XP · ${slot.points} ${labels.points}`));
      const actions = el("div", "dom-slot-actions");
      actions.appendChild(button(labels.load, "small primary", () => uiAction(UiAction.MenuStart, slot.slot, 0)));
      actions.appendChild(button(labels.delete, "small danger", () => uiAction(UiAction.MenuRequestDelete, slot.slot)));
      card.appendChild(actions);
    } else {
      card.appendChild(el("div", "dom-slot-meta muted", labels.empty_slot));
      const actions = el("div", "dom-slot-actions");
      actions.appendChild(button(labels.new_game, "small primary", () => uiAction(UiAction.MenuStart, slot.slot, 1)));
      card.appendChild(actions);
    }
    panel.appendChild(card);
  }
  panel.appendChild(button(labels.back, "secondary", () => uiAction(UiAction.MenuBack)));
}

function renderOptions(panel, state) {
  const labels = state.labels;
  const settings = state.settings;
  panel.appendChild(el("h2", "dom-title", labels.options_title));

  const langRow = el("div", "dom-option-row");
  langRow.appendChild(el("div", "dom-option-label", labels.language));
  const langBtns = el("div", "dom-option-buttons");
  langBtns.appendChild(button("PT-BR", settings.language === "pt-BR" ? "small selected" : "small",
    () => uiAction(UiAction.SetLanguage, 0)));
  langBtns.appendChild(button("EN", settings.language === "en" ? "small selected" : "small",
    () => uiAction(UiAction.SetLanguage, 1)));
  langRow.appendChild(langBtns);
  panel.appendChild(langRow);

  const mcRow = el("div", "dom-option-row");
  mcRow.appendChild(el("div", "dom-option-label", labels.mobile_controls));
  const mcBtns = el("div", "dom-option-buttons");
  const modes = [["on", labels.on, 1], ["off", labels.off, 0], ["auto", labels.auto, 2]];
  for (const [code, label, value] of modes) {
    mcBtns.appendChild(button(label, settings.mobile_controls === code ? "small selected" : "small",
      () => uiAction(UiAction.SetMobileControls, value)));
  }
  mcRow.appendChild(mcBtns);
  panel.appendChild(mcRow);

  const scaleRow = el("div", "dom-option-row");
  scaleRow.appendChild(el("div", "dom-option-label", `${labels.interface_size}: ${Math.round(settings.interface_scale * 100)}%`));
  const scaleBtns = el("div", "dom-option-buttons");
  scaleBtns.appendChild(button("-", "small", () => {
    const pct = Math.round(Math.max(75, (settings.interface_scale - 0.25) * 100));
    uiAction(UiAction.SetInterfaceScale, Math.floor(pct / 256), pct % 256);
  }));
  scaleBtns.appendChild(button("+", "small", () => {
    const pct = Math.round(Math.min(200, (settings.interface_scale + 0.25) * 100));
    uiAction(UiAction.SetInterfaceScale, Math.floor(pct / 256), pct % 256);
  }));
  scaleRow.appendChild(scaleBtns);
  panel.appendChild(scaleRow);

  panel.appendChild(button(labels.back, "secondary", () => uiAction(UiAction.MenuBack)));
}

function renderDeleteModal(root, menu, labels) {
  const overlay = el("div", "dom-modal-overlay");
  const modal = el("div", "dom-modal");
  modal.appendChild(el("h3", "dom-title", `${labels.slot} ${(menu.pending_delete ?? 0) + 1}`));
  modal.appendChild(el("p", "", labels.delete_confirm));
  const actions = el("div", "dom-modal-actions");
  actions.appendChild(button(labels.cancel, "secondary", () => uiAction(UiAction.MenuCancelDelete)));
  actions.appendChild(button(labels.confirm, "danger", () => uiAction(UiAction.MenuConfirmDelete)));
  modal.appendChild(actions);
  overlay.appendChild(modal);
  root.appendChild(overlay);
}

function renderMenus(state) {
  const root = ensureMenuRoot();
  if (!root) return;

  const key = menuKey(state);
  if (key === lastMenuKey) return;
  lastMenuKey = key;

  const showMenu = state && (state.scene === "menu" || state.scene === "menu_over_game") && state.menu;
  if (!showMenu) {
    root.replaceChildren();
    root.className = "dom-menu-root";
    root.style.pointerEvents = "none";
    return;
  }

  root.className = "dom-menu-root active";
  root.style.pointerEvents = "auto";

  const wrap = el("div", "dom-menu-wrap");
  const panel = el("div", "dom-panel");
  const labels = state.labels || {};
  const menu = state.menu;

  if (menu.screen === "main") renderMain(panel, menu, labels);
  else if (menu.screen === "saves") renderSaves(panel, menu, labels);
  else if (menu.screen === "options") renderOptions(panel, state);
  else renderMain(panel, menu, labels);

  wrap.appendChild(panel);
  root.replaceChildren(wrap);

  if (menu.pending_delete != null) {
    renderDeleteModal(root, menu, labels);
  }
}

export function initMenus() {
  subscribe(renderMenus);
}
