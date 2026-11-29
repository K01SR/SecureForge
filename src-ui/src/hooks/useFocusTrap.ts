import { useEffect, useRef } from 'react';

// Traps keyboard focus inside a modal so Tab/Shift+Tab navigation and
// Escape-to-close don't escape the dialog (accessibility + keyboard
// hijacking hardening). Provide a ref to the modal container.
export function useFocusTrap<T extends HTMLElement>(onEscape?: () => void) {
  const ref = useRef<T | null>(null);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && onEscape) {
        e.preventDefault();
        onEscape();
        return;
      }
      if (e.key !== 'Tab') return;

      const focusables = node.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusables.length === 0) return;

      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      const active = document.activeElement as HTMLElement | null;

      if (e.shiftKey) {
        if (active === first || !node.contains(active)) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (active === last || !node.contains(active)) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    node.addEventListener('keydown', handleKeyDown);
    // Focus the first focusable element on open (explicitly after render).
    const firstFocusable = node.querySelector<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (firstFocusable) firstFocusable.focus();

    return () => node.removeEventListener('keydown', handleKeyDown);
  }, [onEscape]);

  return ref;
}
