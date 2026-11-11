// Hash routes, so the site works from any static host and any sub-path.
import { useSyncExternalStore } from 'react';

export type Route =
  | { name: 'home' }
  | { name: 'mission'; id: string }
  | { name: 'concept'; id: string }
  | { name: 'settings' }
  | { name: 'credits' };

export function parseRoute(hash: string): Route {
  const [, name, id] = hash.replace(/^#/, '').split('/');
  if (name === 'mission' && id) return { name, id: decodeURIComponent(id) };
  if (name === 'concept' && id) return { name, id: decodeURIComponent(id) };
  if (name === 'settings' || name === 'credits') return { name };
  return { name: 'home' };
}

export const href = {
  home: '#/',
  mission: (id: string) => `#/mission/${encodeURIComponent(id)}`,
  concept: (id: string) => `#/concept/${encodeURIComponent(id)}`,
  settings: '#/settings',
  credits: '#/credits',
};

function subscribe(listener: () => void) {
  addEventListener('hashchange', listener);
  return () => removeEventListener('hashchange', listener);
}

export function useRoute(): Route {
  const hash = useSyncExternalStore(subscribe, () => location.hash);
  return parseRoute(hash);
}
