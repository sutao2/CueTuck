// Shared by workbench dialogs only; the independent launcher is unaffected.
const states = new WeakMap();
const selector = 'button, input, textarea, select, a[href], [tabindex]';
function controls(element) {
  return [...element.querySelectorAll(selector)].filter((node) =>
    !node.matches(':disabled, [tabindex="-1"], [type="hidden"]') &&
    !node.closest('[hidden], [inert]') && getComputedStyle(node).display !== 'none' &&
    getComputedStyle(node).visibility !== 'hidden',
  );
}

export const vDialogFocus = {
  mounted(element, binding) {
    const state = { previous: document.activeElement, close: binding.value };
    element.tabIndex = -1;
    state.keydown = (event) => {
      const dialogs = document.querySelectorAll('[aria-modal="true"]');
      if (dialogs[dialogs.length - 1] !== element || event.isComposing) return;
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        state.close?.();
      } else if (event.key === 'Tab') {
        const items = controls(element);
        const first = items[0] ?? element;
        const last = items[items.length - 1] ?? element;
        if (!items.length || !items.includes(document.activeElement) ||
          (event.shiftKey ? document.activeElement === first : document.activeElement === last)) {
          event.preventDefault();
          (event.shiftKey ? last : first).focus();
        }
      }
    };
    states.set(element, state);
    element.addEventListener('keydown', state.keydown);
    queueMicrotask(() => {
      if (!element.isConnected) return;
      const items = controls(element);
      (items.find((node) => node.hasAttribute('data-dialog-autofocus')) ??
        items.find((node) => node.matches('input:not([type="file"]), textarea')) ?? element).focus();
    });
  },
  updated(element, binding) { states.get(element).close = binding.value; },
  unmounted(element) {
    const state = states.get(element);
    element.removeEventListener('keydown', state.keydown);
    queueMicrotask(() => {
      if (state.previous?.isConnected &&
        (!document.querySelector('[aria-modal="true"]') || state.previous.closest('[aria-modal="true"]'))) {
        state.previous.focus();
      }
    });
    states.delete(element);
  },
};
