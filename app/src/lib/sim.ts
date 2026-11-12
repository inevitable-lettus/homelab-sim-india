// The only module that talks to the Rust core. The UI renders what comes back
// and never decides validity itself.
import init, {
  add_catalog_json,
  evaluate_json,
  grade_json,
  resolve_json,
  set_bundle,
  validate_world_json,
  wireguard_public_key,
} from '../wasm-pkg/sim_wasm';
import type { Catalog } from './packs';
import type { Artifacts, Bundle, Depth, Grade, Id, Report, Resolved, Target, World } from './types';

let loading: Promise<Bundle> | undefined;

/** Start the core and load the data bundle. Safe to call many times. */
export function loadSim(): Promise<Bundle> {
  loading ??= (async () => {
    const [, response] = await Promise.all([init(), fetch(`${import.meta.env.BASE_URL}data/bundle.json`)]);
    if (!response.ok) throw new Error(`Data bundle returned HTTP ${response.status}`);
    const text = await response.text();
    set_bundle(text);
    return JSON.parse(text) as Bundle;
  })();
  return loading;
}

let catalog: Promise<Catalog> | undefined;

/** Fetch the sandbox catalog and hand it to the core. Only the sandbox calls this. */
export function loadCatalog(): Promise<Catalog> {
  catalog ??= (async () => {
    await loadSim();
    const response = await fetch(`${import.meta.env.BASE_URL}data/catalog.json`);
    if (!response.ok) throw new Error(`Catalog returned HTTP ${response.status}`);
    const text = await response.text();
    add_catalog_json(text);
    // Packs that missions use ship in the mission bundle. The sandbox offers both.
    const base = (await loadSim()) as unknown as Partial<Catalog>;
    const extra = JSON.parse(text) as Catalog;
    const both = <T,>(a: Record<string, T> | undefined, b: Record<string, T> | undefined) => ({ ...a, ...b });
    return {
      ...extra,
      devices: both(base.devices, extra.devices),
      os: both(base.os, extra.os),
      runtimes: both(base.runtimes, extra.runtimes),
      services: both(base.services, extra.services),
      tariffs: both(base.tariffs, extra.tariffs),
      outages: both(base.outages, extra.outages),
      media: both(base.media, extra.media),
    };
  })();
  catalog.catch(() => (catalog = undefined));
  return catalog;
}

export interface WorldProblem {
  kind: 'parse' | 'duplicate-id' | 'unknown-node' | 'unknown-pack' | 'unknown-service' | 'loop' | 'too-big';
  at: string;
}

/** What stops a build from being loaded, as the core sees it. Empty means it can be. */
export function validateWorld(worldJson: string): WorldProblem[] {
  return JSON.parse(validate_world_json(worldJson)) as WorldProblem[];
}

// Everything below is synchronous and needs `loadSim()` to have resolved.

export function evaluate(world: World): Report {
  return JSON.parse(evaluate_json(JSON.stringify(world))) as Report;
}

export function grade(mission: Id, depth: Depth, world: World, artifacts: Artifacts = { files: {} }): Grade {
  return JSON.parse(
    grade_json(mission, JSON.stringify(depth), JSON.stringify(world), JSON.stringify(artifacts)),
  ) as Grade;
}

export function resolveTarget(world: World, target: Target): Resolved {
  return JSON.parse(resolve_json(JSON.stringify(world), JSON.stringify(target))) as Resolved;
}

/** What `wg pubkey` prints for this private key, or null when it is not a key. */
export function publicKeyOf(privateKey: string): string | null {
  return wireguard_public_key(privateKey) ?? null;
}
