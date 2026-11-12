// The only module that talks to the Rust core. The UI renders what comes back
// and never decides validity itself.
import init, { evaluate_json, grade_json, resolve_json, set_bundle, wireguard_public_key } from '../wasm-pkg/sim_wasm';
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
