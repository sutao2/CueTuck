// Page navigation has no Tab trap; confirmation dialogs keep their own focus scope.
const states = new WeakMap();
function visible(element) {
  for (let node = element; node; node = node.parentElement) {
    if (node.hidden || node.hasAttribute('inert') || getComputedStyle(node).display === 'none') return false;
  }
  return element.isConnected;
}
export const vPageFocus = {
  mounted(element, binding) {
    const state = { previous: document.activeElement, close: binding.value };
    element.tabIndex = -1;
    state.keydown = event => {
      if (event.key !== 'Escape' || event.isComposing || event.defaultPrevented || document.querySelector('[aria-modal="true"]')) return;
      event.preventDefault();
      event.stopPropagation();
      state.close?.();
    };
    states.set(element, state);
    element.addEventListener('keydown', state.keydown);
    queueMicrotask(() => {
      if (!visible(element)) return;
      (element.querySelector('[data-dialog-autofocus], input:not(:disabled):not([type="file"]), textarea:not(:disabled)') ?? element).focus();
    });
  },
  updated(element, binding) {
    states.get(element).close = binding.value;
    queueMicrotask(() => {
      const active = document.activeElement;
      if (visible(element) && !document.querySelector('[aria-modal="true"]') &&
        (active === document.body || (element.contains(active) && active.matches(':disabled')))) element.focus();
    });
  },
  unmounted(element) {
    const state = states.get(element);
    element.removeEventListener('keydown', state.keydown);
    queueMicrotask(() => { if (state.previous && visible(state.previous)) state.previous.focus(); });
    states.delete(element);
  },
};
