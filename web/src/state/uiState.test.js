import { beforeEach, describe, expect, it, vi } from "vitest";

const readUiStateJson = vi.fn();
const flushPending = vi.fn();
vi.mock("../bridge/bridge.js", () => ({ readUiStateJson, flushPending }));

describe("uiState polling", () => {
  beforeEach(() => {
    vi.resetModules();
    readUiStateJson.mockReset();
    flushPending.mockReset();
  });

  it("updates state when UiState JSON changes", async () => {
    readUiStateJson
      .mockReturnValueOnce(JSON.stringify({ scene: "menu", menu: { screen: "main" } }))
      .mockReturnValueOnce(JSON.stringify({ scene: "menu", menu: { screen: "main" } }))
      .mockReturnValueOnce(JSON.stringify({ scene: "game", player: { level: 2 } }));

    const mod = await import("./uiState.js");
    mod.pollState();
    expect(mod.uiState.value.scene).toBe("menu");
    mod.pollState();
    expect(mod.uiState.value.scene).toBe("menu"); // unchanged JSON
    mod.pollState();
    expect(mod.uiState.value.scene).toBe("game");
    expect(flushPending).toHaveBeenCalledTimes(3);
  });
});
