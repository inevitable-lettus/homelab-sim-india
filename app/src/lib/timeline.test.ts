import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { grade_json, initSync, set_bundle } from '../wasm-pkg/sim_wasm';
import { duration } from './motion';
import { buildTimeline, restOf, segmentAt, stepAt } from './timeline';
import type { Bundle, Grade, Trace, World } from './types';
import { saveForward, setDial, setJoined } from './world';

const text = readFileSync(resolve('public/data/bundle.json'), 'utf8');
const bundle = JSON.parse(text) as Bundle;

function trace(missionId: string, world: World, attempt = 'open-jellyfin'): Trace {
  const result = JSON.parse(grade_json(missionId, '"guided"', JSON.stringify(world), '{"files":{}}')) as Grade;
  const found = result.report.attempts.find((t) => t.attempt === attempt);
  if (!found) throw new Error('no attempt');
  return found;
}

function world(id: string): World {
  const mission = bundle.missions[id];
  if (!mission) throw new Error(`no mission ${id}`);
  return mission.world;
}

describe('buildTimeline', () => {
  beforeAll(() => {
    initSync({ module: readFileSync(resolve('src/wasm-pkg/sim_wasm_bg.wasm')) });
    set_bundle(text);
  });

  it('keeps every step of every leg, in the order the core gave', () => {
    for (const mission of Object.values(bundle.missions)) {
      for (const attempt of mission.world.attempts) {
        const t = trace(mission.id, mission.world, attempt.id);
        const timeline = buildTimeline(t, mission.world.links, duration);
        expect(timeline.steps.map((s) => s.step)).toEqual(t.legs.flatMap((leg) => leg.steps));
        expect(timeline.legs.map((leg) => leg.id)).toEqual(t.legs.filter((l) => l.steps.length).map((l) => l.id));
      }
    }
  });

  it('is gapless, ordered and within the cap', () => {
    let w = setJoined(setJoined(world('dial-out'), 'server', 'mesh', true), 'phone', 'mesh', true);
    w = setDial(w, 'open-jellyfin', { kind: 'overlay_of', node: 'server', network: 'mesh' });
    const timeline = buildTimeline(trace('dial-out', w), w.links, duration);
    expect(timeline.legs.map((leg) => leg.role)).toEqual(['setup', 'setup', 'request', 'reply']);
    let t = 0;
    for (const segment of timeline.segments) {
      expect(segment.start).toBeCloseTo(t, 6);
      expect(segment.end).toBeGreaterThan(segment.start);
      t = segment.end;
    }
    expect(timeline.total).toBeCloseTo(t, 6);
    expect(timeline.total).toBeLessThanOrEqual(duration.cap + 1e-6);
  });

  it('makes a hop last in proportion to the time the core computed', () => {
    const w = world('the-wall');
    const timeline = buildTimeline(trace('the-wall', w), w.links, duration);
    const hops = timeline.segments.filter((s) => s.kind === 'hop');
    const radio = hops.find((s) => s.kind === 'hop' && s.link === 'phone-mobile');
    const backbone = hops.find((s) => s.kind === 'hop' && s.link === 'mobile-internet');
    if (!radio || !backbone) throw new Error('missing hop');
    // The radio hop is the slowest by far, so it gets the longest time.
    expect(radio.end - radio.start).toBeCloseTo(duration.hopMax, 6);
    expect(backbone.end - backbone.start).toBeLessThan(radio.end - radio.start);
    expect(backbone.end - backbone.start).toBeGreaterThanOrEqual(duration.hopMin);
  });

  it('stops a dropped packet at the door it came in by', () => {
    const w = world('the-wall');
    const timeline = buildTimeline(trace('the-wall', w), w.links, duration);
    const last = timeline.segments.at(-1);
    expect(last).toMatchObject({ kind: 'dwell', from: { node: 'isp', link: 'isp-internet' }, to: { node: 'isp', link: 'isp-internet' } });
    expect(timeline.steps.some((s) => s.step.at === 'home-router')).toBe(false);
  });

  it('travels each cable in the right direction', () => {
    let w = setDial(world('open-a-door'), 'open-jellyfin', { kind: 'public_of', node: 'home-router' });
    w = saveForward(w, 'home-router', { id: 'f', enabled: true, proto: 'tcp', external_port: 8096, to_ip: '192.168.1.20', internal_port: 8096 });
    const timeline = buildTimeline(trace('open-a-door', w), w.links, duration);
    const hop = (leg: number, link: string) =>
      timeline.segments.find((s) => s.kind === 'hop' && s.leg === leg && s.link === link);
    // home-isp is declared router to ISP: the request crosses it backwards, the reply forwards.
    expect(hop(0, 'home-isp')).toMatchObject({ reverse: true });
    expect(hop(1, 'home-isp')).toMatchObject({ reverse: false });
  });

  it('gives a trace with no legs an empty timeline', () => {
    const w = setDial(world('dial-out'), 'open-jellyfin', { kind: 'overlay_of', node: 'server', network: 'mesh' });
    const t = trace('dial-out', w);
    expect(t.legs).toEqual([]);
    expect(buildTimeline(t, w.links, duration)).toMatchObject({ total: 0, steps: [], segments: [] });
  });

  it('is the same every time', () => {
    const w = world('which-address');
    const t = trace('which-address', w, 'check-ip');
    expect(buildTimeline(t, w.links, duration)).toEqual(buildTimeline(t, w.links, duration));
  });

  it('finds the step and segment for a time', () => {
    const w = world('which-address');
    const timeline = buildTimeline(trace('which-address', w, 'check-ip'), w.links, duration);
    expect(stepAt(timeline, -1)).toBe(-1);
    expect(stepAt(timeline, 0)).toBe(0);
    expect(stepAt(timeline, timeline.total)).toBe(timeline.steps.length - 1);
    for (let i = 0; i < timeline.steps.length; i++) {
      const rest = restOf(timeline, i);
      expect(stepAt(timeline, rest)).toBe(i);
      expect(segmentAt(timeline, rest)).toMatchObject({ kind: 'dwell', step: i });
    }
  });
});
