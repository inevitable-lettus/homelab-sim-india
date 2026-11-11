import { useSyncExternalStore } from 'react';

/** Follows a media query. */
export function useMedia(query: string): boolean {
  return useSyncExternalStore(
    (listener) => {
      const list = matchMedia(query);
      list.addEventListener('change', listener);
      return () => list.removeEventListener('change', listener);
    },
    () => matchMedia(query).matches,
  );
}
