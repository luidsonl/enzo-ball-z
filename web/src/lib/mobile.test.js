import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../bridge/bridge.js", () => ({
  MobileMode: { Off: 0, On: 1, Auto: 2 },
  getMobileControlsMode: vi.fn(() => 2),
}));

import { mobileControlsVisible } from "./mobile.js";
import { getMobileControlsMode } from "../bridge/bridge.js";

afterEach(() => {
  vi.mocked(getMobileControlsMode).mockReturnValue(2);
});

describe("mobileControlsVisible", () => {
  it("hides controls in menu scenes", () => {
    expect(mobileControlsVisible({ mobile_controls: "on" }, "menu")).toBe(false);
    expect(mobileControlsVisible({ mobile_controls: "on" }, "menu_over_game")).toBe(false);
  });

  it("respects explicit on/off settings", () => {
    expect(mobileControlsVisible({ mobile_controls: "on" }, "game")).toBe(true);
    expect(mobileControlsVisible({ mobile_controls: "off" }, "game")).toBe(false);
  });

  it("uses bridge mode for auto/unknown settings", () => {
    vi.mocked(getMobileControlsMode).mockReturnValue(1);
    expect(mobileControlsVisible({ mobile_controls: "auto" }, "game")).toBe(true);
    vi.mocked(getMobileControlsMode).mockReturnValue(0);
    expect(mobileControlsVisible({}, "game")).toBe(false);
  });
});
