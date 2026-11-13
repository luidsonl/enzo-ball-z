import { shallowRef } from "vue";
import { flushPending, focusGameCanvas, readUiStateJson } from "../bridge/bridge.js";

export const uiState = shallowRef(null);
let lastJson = null;
let lastScene = null;
let timer = null;

export function getState() {
  return uiState.value;
}

export function pollState() {
  flushPending();
  const json = readUiStateJson();
  if (!json || json === lastJson) return;
  try {
    const next = JSON.parse(json);
    const nextScene = next?.scene ?? null;
    if (nextScene === "game" && lastScene !== "game") {
      focusGameCanvas();
    }
    uiState.value = next;
    lastJson = json;
    lastScene = nextScene;
  } catch {}
}

export function startStatePolling(intervalMs = 150) {
  if (timer) return;
  pollState();
  timer = setInterval(pollState, intervalMs);
}

export function stopStatePolling() {
  if (!timer) return;
  clearInterval(timer);
  timer = null;
}
