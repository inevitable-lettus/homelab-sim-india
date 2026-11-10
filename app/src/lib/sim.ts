// The only module that talks to the Rust core. The UI renders what comes back
// and never decides validity itself.
import init, { evaluate_json } from '../wasm-pkg/sim_wasm';

export type Status = 'pass' | 'fail' | 'unknown';

export interface Finding {
  rule_id: string;
  status: Status;
  severity: 'error' | 'warning' | 'info';
  involved: string[];
  explanation_id: string;
  provenance: 'official' | 'community' | 'teaching' | 'line_check';
}

export interface Report {
  schema_version: number;
  findings: Finding[];
}

export interface World {
  nodes: { id: string }[];
}

let ready: Promise<unknown> | undefined;

export async function evaluate(world: World, bundleJson: string): Promise<Report> {
  ready ??= init();
  await ready;
  return JSON.parse(evaluate_json(JSON.stringify(world), bundleJson)) as Report;
}

/** Proves the data bundle and the WASM core both reach the browser. */
export async function runEngineCheck(): Promise<Report> {
  const response = await fetch(`${import.meta.env.BASE_URL}data/bundle.json`);
  if (!response.ok) throw new Error(`Data bundle returned HTTP ${response.status}`);
  return evaluate({ nodes: [] }, await response.text());
}
