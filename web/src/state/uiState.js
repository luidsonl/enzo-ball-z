import { shallowRef } from "vue";
import { flushPending, readUiStateJson } from "../bridge/bridge.js";

export const uiState = shallowRef(null);
let lastJson = null;
let timer = null;

export function getState() {
  return uiState.value;
}

export function pollState() {
  flushPending();
  const json = readUiStateJson();
  if (!json || json === lastJson) return;
  try {
    uiState.value = JSON.parse(json);
    lastJson = json;
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
