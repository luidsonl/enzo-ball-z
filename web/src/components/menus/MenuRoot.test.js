import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";

const mocks = vi.hoisted(() => ({ uiAction: vi.fn() }));
vi.mock("../../bridge/bridge.js", async (importOriginal) => {
  const actual = await importOriginal();
  return { ...actual, uiAction: mocks.uiAction };
});

import MenuRoot from "./MenuRoot.vue";
import { UiAction } from "../../bridge/bridge.js";

describe("MenuRoot", () => {
  it("renders saves screen and emits semantic actions", async () => {
    mocks.uiAction.mockClear();
    const wrapper = mount(MenuRoot, {
      props: {
        state: {
          scene: "menu",
          labels: { saves_title: "Saves", choose_slot: "Choose", slot: "Slot", empty_slot: "Empty", new_game: "New Game", load: "Load", delete: "Delete", back: "Back", level: "Lv", points: "pts" },
          settings: {},
          menu: {
            screen: "saves",
            has_background_game: false,
            pending_delete: null,
            slots: [{ slot: 0, occupied: false }, { slot: 1, occupied: true, level: 3, xp: 10, xp_to_next: 300, points: 1 }],
          },
        },
      },
    });

    expect(wrapper.text("#dom-menu-root")).toContain("Saves");
    const buttons = wrapper.findAll("button");
    await buttons.find((b) => b.text() === "New Game").trigger("click");
    await buttons.find((b) => b.text() === "Load").trigger("click");
    await buttons.find((b) => b.text() === "Back").trigger("click");

    expect(mocks.uiAction).toHaveBeenCalledWith(UiAction.MenuStart, 0, 1, 0);
    expect(mocks.uiAction).toHaveBeenCalledWith(UiAction.MenuStart, 1, 0, 0);
    expect(mocks.uiAction).toHaveBeenCalledWith(UiAction.MenuBack, 0, 0, 0);
  });
});
