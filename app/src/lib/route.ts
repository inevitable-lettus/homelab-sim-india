// Hash routes, so the site works from any static host and any sub-path.
import { useSyncExternalStore } from 'react';

export type Route =
  | { name: 'home' }
  | { name: 'mission'; id: string }
  | { name: 'concept'; id: string }
  | { name: 'builds' }
  | { name: 'build'; id: string }
  | { name: 'buildImport'; payload: string }
  | { name: 'settings' }
  | { name: 'credits' };

export function parseRoute(hash: string): Route {
  const [, name, id, rest] = hash.replace(/^#/, '').split('/');
  if (name === 'build' && id === 'import' && rest) return { name: 'buildImport', payload: rest };
  if (name === 'build' && id) return { name: 'build', id: decodeURIComponent(id) };
  if (name === 'build') return { name: 'builds' };
  if (name === 'mission' && id) return { name, id: decodeURIComponent(id) };
  if (name === 'concept' && id) return { name, id: decodeURIComponent(id) };
  if (name === 'settings' || name === 'credits') return { name };
  return { name: 'home' };
}

export const href = {
  home: '#/',
  mission: (id: string) => `#/mission/${encodeURIComponent(id)}`,
  concept: (id: string) => `#/concept/${encodeURIComponent(id)}`,
  builds: '#/build',
  build: (id: string) => `#/build/${encodeURIComponent(id)}`,
  buildImport: (payload: string) => `#/build/import/${payload}`,
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
