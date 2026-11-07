import { UiAction, uiAction } from "./bridge.js";
import { mobileControlsVisible } from "./mobile-controls.js";
import { subscribe } from "./state.js";

let lastHudKey = null;

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
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

function hudKey(state) {
  if (!state || (state.scene !== "game" && state.scene !== "menu_over_game")) return "hidden";
  const mobile = mobileControlsVisible(state?.settings) ? 1 : 0;
  if (!state.player) return `hidden:${state.scene}:${state.phase ?? ""}:${state.game_toast ?? ""}:${mobile}`;
  const p = state.player;
  const slots = (p.slots || []).map(s => `${s.slot}:${s.skill ?? ""}:${s.available?1:0}:${s.active?1:0}:${s.cooldown.toFixed(1)}`).join("|");
  return [state.scene, state.phase ?? "", state.game_toast ?? "", mobile, Math.ceil(p.health), Math.ceil(p.max_health), Math.ceil(p.aura), Math.ceil(p.max_aura), p.level, Math.floor(p.xp), p.points, slots, p.attributes?.speed, p.attributes?.strength, p.attributes?.health, p.attributes?.aura].join("::");
}

function renderHud(state) {
  const root = ensureHudRoot();
  if (!root) return;

  const key = hudKey(state);
  if (key === lastHudKey) return;
  lastHudKey = key;

  const inGameScene = state && (state.scene === "game" || state.scene === "menu_over_game");
  if (!inGameScene) {
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
    top.appendChild(bars);

    if (state.scene === "game") {
      const menuBtn = document.createElement("button");
      menuBtn.type = "button";
      menuBtn.className = "dom-hud-menu-btn";
      menuBtn.textContent = labels.menu || "Menu";
      menuBtn.style.pointerEvents = "auto";
      menuBtn.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudOpenMenu); });
      top.appendChild(menuBtn);
    }
    frag.appendChild(top);

    // When mobile controls are visible, slot buttons are provided by mobile controls.
    if (!mobileControlsVisible(state?.settings)) {
      const actionBar = el("div", "dom-action-bar");
      actionBar.style.pointerEvents = state.scene === "game" ? "auto" : "none";
      (player.slots || []).forEach((slot, index) => {
        const b = document.createElement("button");
        b.type = "button";
        b.className = "dom-action-slot";
        b.dataset.slot = String(index);
        if (slot.active) b.classList.add("active");
        if (!slot.available) b.classList.add("locked");
        if (slot.cooldown > 0.01) b.classList.add("cooldown");
        b.innerHTML = `<span class="slot-num">${index + 1}</span><span class="slot-skill">${slot.skill != null ? `S${slot.skill}` : "—"}</span>`;
        if (slot.cooldown > 0.01 && slot.cooldown_max > 0) b.appendChild(el("span", "slot-cd", slot.cooldown.toFixed(1)));
        b.title = slot.lock_reason || "";
        b.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudOpenSkills); });
        actionBar.appendChild(b);
      });
      frag.appendChild(actionBar);
    }
  }

  if (state.game_toast) {
    frag.appendChild(el("div", "dom-game-toast", state.game_toast));
  }

  if (phase === "Dead" || phase === "Victory" || phase === "LevelUp") {
    const overlay = el("div", "dom-phase-overlay");
    const panel = el("div", "dom-phase-panel");
    const title = phase === "Dead" ? (labels.game_over || "Game Over")
      : phase === "Victory" ? (labels.victory || "Victory")
      : (labels.level_up || "Level Up");
    panel.appendChild(el("h2", "dom-title", title));

    if (phase === "LevelUp" && player && player.points > 0) {
      panel.appendChild(el("p", "dom-subtitle", `${labels.points || "Points"}: ${player.points}`));
      const attrs = el("div", "dom-attr-grid");
      const names = ["Speed", "Strength", "Health", "Aura"];
      const values = [player.attributes?.speed, player.attributes?.strength, player.attributes?.health, player.attributes?.aura];
      names.forEach((name, i) => {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "dom-btn small";
        btn.textContent = `+ ${name} (${values[i] ?? 0})`;
        btn.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudAllocate, i); });
        attrs.appendChild(btn);
      });
      panel.appendChild(attrs);
    }

    const actions = el("div", "dom-modal-actions");
    if (phase === "LevelUp") {
      const c = document.createElement("button"); c.type = "button"; c.className = "dom-btn primary"; c.textContent = labels.confirm || "Confirm";
      c.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudConfirmLevelUp); });
      actions.appendChild(c);
    } else {
      const r = document.createElement("button"); r.type = "button"; r.className = "dom-btn primary"; r.textContent = labels.restart || "Restart";
      r.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudRestart); });
      actions.appendChild(r);
      const m = document.createElement("button"); m.type = "button"; m.className = "dom-btn secondary"; m.textContent = labels.menu || "Menu";
      m.addEventListener("click", (e) => { e.preventDefault(); e.stopPropagation(); uiAction(UiAction.HudOpenMenu); });
      actions.appendChild(m);
    }
    panel.appendChild(actions);
    overlay.appendChild(panel);
    frag.appendChild(overlay);
  }

  root.replaceChildren(frag);
}

export function initHud() {
  subscribe(renderHud);
}
