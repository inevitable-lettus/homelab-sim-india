// Everything the app remembers lives in this browser's localStorage and
// nowhere else. Nothing here is ever sent anywhere.
import { useSyncExternalStore } from 'react';
import type { Depth, Id, World } from './types';

const KEY = 'homelab-sim:v1';

export type MotionSetting = 'system' | 'reduced' | 'full';
export type ThemeSetting = 'system' | 'light' | 'dark';

export interface Saved {
  version: 1;
  depth: Depth;
  motion: MotionSetting;
  theme: ThemeSetting;
  /** Mission id to the date each depth was finished. */
  done: Record<Id, Partial<Record<Depth, string>>>;
  /** The learner's board for a mission they have started. */
  drafts: Record<Id, World>;
}

const fresh = (): Saved => ({ version: 1, depth: 'guided', motion: 'system', theme: 'system', done: {}, drafts: {} });

function read(): Saved {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return fresh();
    const parsed = JSON.parse(raw) as Partial<Saved>;
    return parsed.version === 1 ? { ...fresh(), ...parsed } : fresh();
  } catch {
    return fresh();
  }
}

let state = read();
const listeners = new Set<() => void>();

function write(next: Saved) {
  state = next;
  try {
    localStorage.setItem(KEY, JSON.stringify(next));
  } catch {
    // Private mode or a full disk. The app still works for this visit.
  }
  listeners.forEach((listener) => listener());
}

export const progress = {
  get: () => state,
  update: (change: Partial<Saved>) => write({ ...state, ...change }),
  finish(mission: Id, depth: Depth) {
    if (state.done[mission]?.[depth]) return;
    const day = new Date().toISOString().slice(0, 10);
    write({ ...state, done: { ...state.done, [mission]: { ...state.done[mission], [depth]: day } } });
  },
  saveDraft(mission: Id, world: World) {
    write({ ...state, drafts: { ...state.drafts, [mission]: world } });
  },
  dropDraft(mission: Id) {
    const drafts = { ...state.drafts };
    delete drafts[mission];
    write({ ...state, drafts });
  },
  clear: () => write(fresh()),
  export: () => JSON.stringify(state, null, 2),
  /** Returns false when the text is not a progress file. */
  import(text: string): boolean {
    try {
      const parsed = JSON.parse(text) as Partial<Saved>;
      if (parsed.version !== 1 || typeof parsed.done !== 'object' || parsed.done === null) return false;
      write({ ...fresh(), ...parsed });
      return true;
    } catch {
      return false;
    }
  },
};

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useProgress(): Saved {
  return useSyncExternalStore(subscribe, progress.get);
}

const reducedQuery = typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;

function subscribeReduced(listener: () => void) {
  reducedQuery?.addEventListener('change', listener);
  return () => reducedQuery?.removeEventListener('change', listener);
}

/** True when packets should not travel: the system asks for it, or the learner did. */
export function useReducedMotion(): boolean {
  const system = useSyncExternalStore(subscribeReduced, () => reducedQuery?.matches ?? false);
  const { motion } = useProgress();
  return motion === 'reduced' || (motion === 'system' && system);
}
