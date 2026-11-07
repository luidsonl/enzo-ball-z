import { afterEach, describe, expect, it, vi } from "vitest";

async function loadBridge() {
  vi.resetModules();
  return await import("./bridge.js");
}

afterEach(() => {
  delete globalThis.wasm_exports;
  delete globalThis.wasm_memory;
});

describe("bridge constants", () => {
  it("keeps append-only action/UI codes stable", async () => {
    const { Action, UiAction, MobileMode } = await loadBridge();
    expect(Action.MoveLeft).toBe(0);
    expect(Action.OpenSkills).toBe(17);
    expect(UiAction.MenuBack).toBe(1);
    expect(UiAction.HudAllocate).toBe(12);
    expect(UiAction.HudOpenSkills).toBe(16);
    expect(MobileMode).toEqual({ Off: 0, On: 1, Auto: 2 });
  });
});

describe("bridge exports readiness and queueing", () => {
  it("reports not ready without wasm exports", async () => {
    const { exportsReady } = await loadBridge();
    expect(exportsReady()).toBe(false);
  });

  it("queues ui and loadout actions until exports are ready, then flushes", async () => {
    const { uiAction, loadoutAssign, loadoutClear, flushPending } = await loadBridge();
    uiAction(12, 3, 0, 0);
    loadoutAssign(1, 7);
    loadoutClear(2);

    const calls = [];
    globalThis.wasm_exports = {
      enzo_action_down() {},
      enzo_action_up() {},
      enzo_ui_action: (...args) => calls.push(["ui", ...args]),
      enzo_loadout_assign: (...args) => calls.push(["assign", ...args]),
      enzo_loadout_clear: (...args) => calls.push(["clear", ...args]),
    };

    flushPending();
    expect(calls).toEqual([
      ["ui", 12, 3, 0, 0],
      ["assign", 1, 7],
      ["clear", 2],
    ]);
  });

  it("calls exports immediately when ready", async () => {
    const calls = [];
    globalThis.wasm_exports = {
      enzo_action_down: (code) => calls.push(["down", code]),
      enzo_action_up: (code) => calls.push(["up", code]),
      enzo_ui_action: (...args) => calls.push(["ui", ...args]),
    };
    const { actionDown, actionUp, uiAction } = await loadBridge();
    actionDown(4);
    actionUp(4);
    uiAction(15);
    expect(calls).toEqual([["down", 4], ["up", 4], ["ui", 15, 0, 0, 0]]);
  });
});
