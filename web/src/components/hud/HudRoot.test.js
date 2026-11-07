import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";

const mocks = vi.hoisted(() => ({
  uiAction: vi.fn(),
  loadoutAssign: vi.fn(),
  loadoutClear: vi.fn(),
}));
vi.mock("../../bridge/bridge.js", async (importOriginal) => {
  const actual = await importOriginal();
  return { ...actual, uiAction: mocks.uiAction, loadoutAssign: mocks.loadoutAssign, loadoutClear: mocks.loadoutClear };
});
vi.mock("../../lib/mobile.js", () => ({ mobileControlsVisible: () => false }));

import HudRoot from "./HudRoot.vue";
import { UiAction } from "../../bridge/bridge.js";

function gameState(overrides = {}) {
  return {
    scene: "game",
    phase: "Playing",
    labels: {
      status: "Status", equipment: "Equipment", skills: "Skills", inventory: "Items",
      points: "Points", level: "Level", menu: "Menu", close: "Close", assign: "Assign", clear: "Clear",
    },
    settings: { language: "en", mobile_controls: "off" },
    player: {
      health: 100, max_health: 100, aura: 80, max_aura: 100,
      level: 2, xp: 10, xp_to_next: 200, points: 1, pending_level_up: true,
      attributes: { speed: 0, strength: 0, health: 0, aura: 0 },
      slots: [
        { slot: 0, skill: 0, skill_name: "Jump", available: true, active: false, cooldown: 0 },
        { slot: 1, skill: 1, skill_name: "Dash", available: true, active: false, cooldown: 0 },
        { slot: 2, skill: 3, skill_name: "Melee", available: true, active: false, cooldown: 0 },
        { slot: 3, skill: 4, skill_name: "Mogging", available: true, active: false, cooldown: 0 },
      ],
      catalog: [
        { skill: 0, name: "Jump", available: true, category: "Movement", equipped_slot: 0, aura_cost: 0, cooldown_max: 0 },
        { skill: 4, name: "Mogging", available: true, category: "Attack", equipped_slot: 3, aura_cost: 25, cooldown_max: 0.35 },
        { skill: 5, name: "Nutella", available: false, category: "Attack", equipped_slot: null, aura_cost: 0, cooldown_max: 0.25 },
      ],
      equipment_slots: [{ slot: "weapon", slot_label: "Weapon", item: null }],
      inventory: [],
    },
    ...overrides,
  };
}

describe("HudRoot", () => {
  it("blinks Status tab and allocates points via HudAllocate", async () => {
    mocks.uiAction.mockClear();
    const wrapper = mount(HudRoot, { props: { state: gameState() } });
    const statusTab = wrapper.findAll("button.dom-hud-tab").find((b) => b.text() === "Status");
    expect(statusTab).toBeTruthy();
    expect(statusTab.classes()).toContain("blink");
    await statusTab.trigger("click");
    await wrapper.vm.$nextTick();

    const speedBtn = wrapper.findAll("button").find((b) => b.text().includes("+ Speed"));
    expect(speedBtn, wrapper.text()).toBeTruthy();
    await speedBtn.trigger("click");
    expect(mocks.uiAction).toHaveBeenCalledWith(UiAction.HudAllocate, 0, 0, 0);
  });

  it("shows only unlocked skills in Skills panel", async () => {
    mocks.loadoutAssign.mockClear();
    const wrapper = mount(HudRoot, { props: { state: gameState() } });
    const skillsTab = wrapper.findAll("button.dom-hud-tab").find((b) => b.text() === "Skills");
    await skillsTab.trigger("click");
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("Jump");
    expect(wrapper.text()).toContain("Mogging");
    expect(wrapper.text()).not.toContain("Nutella");

    const assignButtons = wrapper.findAll("button").filter((b) => b.text() === "Assign");
    expect(assignButtons.length, wrapper.text()).toBeGreaterThan(0);
    await assignButtons[0].trigger("click");
    expect(mocks.loadoutAssign).toHaveBeenCalled();
  });
});
