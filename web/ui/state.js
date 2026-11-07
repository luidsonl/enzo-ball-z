import { flushPending, readUiStateJson } from "./bridge.js";

let state = null;
let lastJson = null;
const listeners = new Set();

export function getState() {
  return state;
}

export function subscribe(listener) {
  listeners.add(listener);
  if (state) {
    try { listener(state); } catch (_) {}
  }
  return () => listeners.delete(listener);
}

function notify() {
  for (const listener of listeners) {
    try { listener(state); } catch (_) {}
  }
}

export function pollState() {
  flushPending();
  const json = readUiStateJson();
  if (!json || json === lastJson) return;
  try {
    state = JSON.parse(json);
    lastJson = json;
    notify();
  } catch (_) {}
}

export function initState() {
  setInterval(pollState, 150);
}
