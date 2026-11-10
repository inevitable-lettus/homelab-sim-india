import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { evaluate_json, grade_json, initSync, resolve_json, set_bundle } from '../wasm-pkg/sim_wasm';
import type { Bundle, Grade, Report, Resolved, World } from './types';

const text = readFileSync(resolve('public/data/bundle.json'), 'utf8');
const bundle = JSON.parse(text) as Bundle;

function mission(id: string) {
  const found = bundle.missions[id];
  if (!found) throw new Error(`no mission ${id}`);
  return found;
}

function grade(id: string, world: World): Grade {
  return JSON.parse(grade_json(id, '"guided"', JSON.stringify(world), '{"files":{}}')) as Grade;
}

describe('rust core through wasm', () => {
  beforeAll(() => {
    initSync({ module: readFileSync(resolve('src/wasm-pkg/sim_wasm_bg.wasm')) });
    set_bundle(text);
  });

  it('reports an empty world as unknown, never a pass', () => {
    const report = JSON.parse(evaluate_json('{"nodes":[],"links":[]}')) as Report;
    expect(report.findings[0]?.status).toBe('unknown');
  });

  it('rejects malformed input instead of guessing', () => {
    expect(() => evaluate_json('not json')).toThrow();
    expect(() => evaluate_json('{"nodes":[],"links":[],"surprise":1}')).toThrow();
  });

  it('refuses a bundle from another schema version', () => {
    expect(() => set_bundle('{"schema_version":1}')).toThrow(/schema/);
    set_bundle(text);
  });

  it('has words for every reason a trace or finding uses', () => {
    for (const m of Object.values(bundle.missions)) {
      const result = grade(m.id, m.world);
      const ids = [
        ...result.goals.map((g) => g.reason),
        ...result.report.findings.flatMap((f) => [f.rule_id, f.explanation_id]),
        ...result.report.attempts.flatMap((t) => [t.reason, ...t.legs.flatMap((l) => l.steps.map((s) => s.reason))]),
      ].filter((id): id is string => id !== null);
      for (const id of ids) expect(bundle.explanations[id]?.line, id).toBeTruthy();
    }
  });

  it('no mission starts solved', () => {
    for (const m of Object.values(bundle.missions)) expect(grade(m.id, m.world).status).toBe('fail');
  });

  it('blocks the port forward behind carrier NAT at the ISP', () => {
    const m = mission('the-wall');
    const trace = grade(m.id, m.world).report.attempts.find((t) => t.attempt === 'open-jellyfin');
    expect(trace?.outcome).toBe('dropped');
    expect(trace?.ended_at).toBe('isp');
    expect(trace?.reason).toBe('cgnat-no-inbound-mapping');
  });

  it('gives identical bytes for identical input', () => {
    const m = mission('dial-out');
    const world = JSON.stringify(m.world);
    expect(grade_json(m.id, '"guided"', world, '{"files":{}}')).toBe(grade_json(m.id, '"guided"', world, '{"files":{}}'));
  });

  it('resolves dial targets from the world', () => {
    const m = mission('the-wall');
    const ask = (target: unknown) => JSON.parse(resolve_json(JSON.stringify(m.world), JSON.stringify(target))) as Resolved;
    expect(ask({ kind: 'lan_of', node: 'server' }).address).toBe('192.168.1.20');
    expect(ask({ kind: 'wan_of', node: 'home-router' }).address).toBe('100.64.12.34');
    expect(ask({ kind: 'public_of', node: 'home-router' }).address).toBe('203.0.113.50');
  });
});
