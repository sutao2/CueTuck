import { onBeforeUnmount } from 'vue';
// Only explicitly selected list controls, never rows, secrets or edit drafts.
const states = new Map();
let generation = 0;
export function clearListStates() { states.clear(); generation++; }
export function listState(page) {
  const current = generation;
  return {
    read: () => structuredClone(states.get(page) || {}),
    save(value) { if (current === generation) states.set(page, JSON.parse(JSON.stringify(value))); },
  };
}
export function rememberListControls(page, controls) {
  const memory = listState(page), previous = memory.read();
  for (const [key, control] of Object.entries(controls)) if (key in previous) control.value = previous[key];
  onBeforeUnmount(() => memory.save(Object.fromEntries(Object.entries(controls).map(([key, control]) => [key, control.value]))));
}
