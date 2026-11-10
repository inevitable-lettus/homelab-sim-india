import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { evaluate_json, initSync } from '../wasm-pkg/sim_wasm';
import type { Report } from './sim';

const bundle = readFileSync(resolve('public/data/bundle.json'), 'utf8');

function run(world: unknown): Report {
  return JSON.parse(evaluate_json(JSON.stringify(world), bundle)) as Report;
}

describe('rust core through wasm', () => {
  beforeAll(() => {
    initSync({ module: readFileSync(resolve('src/wasm-pkg/sim_wasm_bg.wasm')) });
  });

  it('reports an empty world as unknown, never a pass', () => {
    expect(run({ nodes: [] }).findings[0]?.status).toBe('unknown');
  });

  it('gives identical output for identical input', () => {
    const world = { nodes: [{ id: 'b' }, { id: 'a' }] };
    expect(run(world)).toEqual(run(world));
  });

  it('rejects malformed input instead of guessing', () => {
    expect(() => evaluate_json('not json', bundle)).toThrow();
  });
});
