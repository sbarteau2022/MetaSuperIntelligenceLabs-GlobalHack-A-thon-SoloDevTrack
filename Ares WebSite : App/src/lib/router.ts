import { useEffect } from 'react';
import { useAtomaStore, type PageId } from '../store/useAtomaStore';
import { logger } from './logger';

const VALID: ReadonlySet<string> = new Set([
  'platform', 'tools-mcp', 'research-lab', 'research-hub', 'meta-hackathon',
  'ide-workbench', 'swarm-legacy', 'contact', 'sign-up', 'downloads', 'cloud-app',
]);

function pageFromHash(): PageId | null {
  const h = window.location.hash.replace(/^#\/?/, '').split('?')[0].trim();
  if (!h) return null;
  return (VALID as Set<string>).has(h) ? (h as PageId) : null;
}

function hashFor(page: PageId): string {
  return `#/${page}`;
}

/**
 * CONTRACT-FIRST deep-linking.
 * - Store remains source of truth for rendering.
 * - Hash is the shareable contract: #/platform, #/cloud-app, …
 * - Unknown hashes → 'platform' + NotFound flag via `?missing=` (rendered by App).
 */
export function useHashRoute() {
  const { activePage, setActivePage } = useAtomaStore();

  // store → URL (replace to avoid history spam on tab switches)
  useEffect(() => {
    const want = hashFor(activePage);
    if (window.location.hash !== want) {
      window.history.replaceState(null, '', want);
    }
    // Announce route for a11y + scroll reset handled by callers.
    document.title = `${routeTitle(activePage)} — Atoma Engineering`;
  }, [activePage]);

  // URL → store (back/forward + direct load + shared links)
  useEffect(() => {
    const apply = () => {
      const p = pageFromHash();
      if (p) {
        if (p !== useAtomaStore.getState().activePage) {
          logger.debug('route', `hash → ${p}`);
          setActivePage(p);
        }
      } else if (window.location.hash && window.location.hash.length > 2) {
        // Unknown route: fall back to platform but preserve intent for 404 UI.
        sessionStorage.setItem('atoma.missing-route', window.location.hash);
        setActivePage('platform');
        window.history.replaceState(null, '', '#/platform');
        (window as unknown as { __ATOMA_404__?: boolean }).__ATOMA_404__ = true;
      }
    };
    apply();
    window.addEventListener('hashchange', apply);
    return () => window.removeEventListener('hashchange', apply);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}

export function routeTitle(page: PageId): string {
  switch (page) {
    case 'cloud-app': return 'Cloud Workspace';
    case 'tools-mcp': return 'Tools & MCP';
    case 'research-lab': return 'Research Lab';
    case 'research-hub': return 'Research Hub';
    case 'meta-hackathon': return 'Meta Hack-a-thon';
    case 'ide-workbench': return 'IDE & DAG Lab';
    case 'contact': return 'Contact';
    case 'sign-up': return 'Sign in';
    case 'downloads': return 'Downloads';
    case 'swarm-legacy': return 'Swarm Legacy';
    default: return 'ARES Spec Ops Platform';
  }
}

export function wasUnknownRoute(): boolean {
  return (window as unknown as { __ATOMA_404__?: boolean }).__ATOMA_404__ === true;
}

export function clearUnknownRoute() {
  (window as unknown as { __ATOMA_404__?: boolean }).__ATOMA_404__ = false;
  sessionStorage.removeItem('atoma.missing-route');
}
