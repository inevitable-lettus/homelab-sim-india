// The Rust core, loaded in Node, so a test can ask it what the page should show.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { Bundle, Grade, Step, World } from '../src/lib/types';
import { grade_json, initSync, set_bundle } from '../src/wasm-pkg/sim_wasm.js';

const text = readFileSync(resolve('public/data/bundle.json'), 'utf8');
export const bundle = JSON.parse(text) as Bundle;

let ready = false;
export function grade(mission: string, world: World): Grade {
  if (!ready) {
    initSync({ module: readFileSync(resolve('src/wasm-pkg/sim_wasm_bg.wasm')) });
    set_bundle(text);
    ready = true;
  }
  return JSON.parse(grade_json(mission, '"guided"', JSON.stringify(world), '{"files":{}}')) as Grade;
}

export function missionWorld(id: string): World {
  const mission = bundle.missions[id];
  if (!mission) throw new Error(`no mission ${id}`);
  return structuredClone(mission.world);
}

/** Every step of an attempt, in order, as the step log should list it. */
export function stepsOf(result: Grade, attempt: string): Pick<Step, 'at' | 'reason'>[] {
  const trace = result.report.attempts.find((t) => t.attempt === attempt);
  if (!trace) throw new Error(`no attempt ${attempt}`);
  return trace.legs.flatMap((leg) => leg.steps.map((step) => ({ at: step.at, reason: step.reason })));
}
