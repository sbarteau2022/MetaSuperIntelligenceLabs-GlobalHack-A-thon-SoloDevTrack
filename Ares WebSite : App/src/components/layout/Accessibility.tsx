import { useEffect } from 'react';

/**
 * Skip-link + route-focus manager.
 * - Skip link is always first in tab order (rendered in main.tsx).
 * - On route change, moves screen-reader focus to #main (tabIndex=-1).
 */
export function SkipLink() {
  return (
    <a
      href="#main"
      className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-[100] focus:px-4 focus:py-2 focus:rounded-lg focus:bg-[#389cff] focus:text-white focus:text-sm focus:font-semibold"
    >
      Skip to content
    </a>
  );
}

export function useRouteFocus(routeKey: string) {
  useEffect(() => {
    const main = document.getElementById('main');
    if (main) {
      // -1 tabIndex set in markup; focus({preventScroll:true}) avoids jump,
      // callers scroll explicitly.
      try {
        main.focus({ preventScroll: true });
      } catch {
        /* older browsers */
      }
    }
  }, [routeKey]);
}
