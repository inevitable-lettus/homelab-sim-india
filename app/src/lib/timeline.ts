// Turns a trace from the core into timed segments for playback. Pure: no DOM,
// no clock. The core decides what happened; this only decides how long each
// part is shown for.
import type { Durations } from './motion';
import type { Id, Leg, LegRole, Link, Step, Trace } from './types';

/** A place on the board: the end of a cable at a node, or the node's centre. */
export interface Port {
  node: Id;
  link?: Id;
}

export interface FlatStep {
  index: number;
  leg: number;
  step: Step;
  /** The step before it in the same leg. */
  previous?: Step;
}

export interface LegSpan {
  id: string;
  role: LegRole;
  from: Id;
  firstStep: number;
  lastStep: number;
  start: number;
  end: number;
  durationNs: number | null;
}

export type Segment =
  /** The packet crosses a cable towards `step`. */
  | { kind: 'hop'; step: number; leg: number; start: number; end: number; link: Id; reverse: boolean; queueShare: number }
  /** The packet is at a node while `step` happens. */
  | { kind: 'dwell'; step: number; leg: number; start: number; end: number; from: Port; to: Port }
  /** Nothing on screen between two legs. */
  | { kind: 'gap'; step: number; leg: number; start: number; end: number };

export interface Timeline {
  steps: FlatStep[];
  legs: LegSpan[];
  segments: Segment[];
  total: number;
  /** When each step's dwell begins and ends. A paused player rests just before the end. */
  stepStart: number[];
  stepEnd: number[];
}

type LinkEnds = Pick<Link, 'id' | 'a' | 'b'>;

const CHANGES = new Set(['rewrite', 'encapsulate', 'decapsulate']);

function dwellFor(step: Step, first: boolean, dur: Durations): number {
  if (step.action === 'drop' || step.action === 'unknown') return dur.drop;
  if (step.action === 'deliver') return dur.deliver;
  if (CHANGES.has(step.action) || step.table) return dur.rewrite;
  if (first) return dur.appear;
  return dur.forward;
}

export function buildTimeline(trace: Trace, links: LinkEnds[], dur: Durations): Timeline {
  const linkById = new Map(links.map((link) => [link.id, link]));
  // A hop's time on screen is its share of the slowest hop in this trace.
  const slowest = Math.max(
    0,
    ...trace.legs.flatMap((leg) => leg.steps.map((step) => step.timing?.total_ns ?? 0)),
  );
  const hopTime = (step: Step) => {
    const ns = step.timing?.total_ns;
    if (ns === null || ns === undefined || slowest === 0) return dur.hop;
    return Math.min(dur.hopMax, Math.max(dur.hopMin, (ns / slowest) * dur.hopMax));
  };

  const steps: FlatStep[] = [];
  const legs: LegSpan[] = [];
  const segments: Segment[] = [];
  const stepStart: number[] = [];
  const stepEnd: number[] = [];
  let t = 0;

  trace.legs.forEach((leg: Leg, legIndex) => {
    if (leg.steps.length === 0) return;
    if (segments.length > 0) {
      segments.push({ kind: 'gap', step: steps.length, leg: legIndex, start: t, end: t + dur.legGap });
      t += dur.legGap;
    }
    const legStart = t;
    const firstStep = steps.length;
    leg.steps.forEach((step, i) => {
      const index = steps.length;
      const previous = leg.steps[i - 1];
      const next = leg.steps[i + 1];
      const arrivedBy = previous && previous.at !== step.at ? step.via_link : undefined;
      const leavesBy = next && next.at !== step.at ? next.via_link : undefined;

      if (previous && arrivedBy) {
        const link = linkById.get(arrivedBy);
        const length = hopTime(step);
        const timing = step.timing;
        const queueShare =
          timing?.queue_ns && timing.total_ns ? Math.min(0.6, timing.queue_ns / timing.total_ns) : 0;
        segments.push({
          kind: 'hop',
          step: index,
          leg: legIndex,
          start: t,
          end: t + length,
          link: arrivedBy,
          reverse: link ? link.a !== previous.at : false,
          queueShare,
        });
        t += length;
      }

      // Where the packet sits while this step happens. A packet that is
      // dropped never gets past the door it came in by.
      const from: Port = arrivedBy ? { node: step.at, link: arrivedBy } : { node: step.at };
      const stops = step.action === 'drop' || step.action === 'unknown';
      const to: Port = stops ? from : leavesBy ? { node: step.at, link: leavesBy } : { node: step.at };
      const length = dwellFor(step, i === 0, dur);
      stepStart.push(t);
      segments.push({ kind: 'dwell', step: index, leg: legIndex, start: t, end: t + length, from, to });
      t += length;
      stepEnd.push(t);
      steps.push({ index, leg: legIndex, step, previous });
    });
    legs.push({
      id: leg.id,
      role: leg.role,
      from: leg.from,
      firstStep,
      lastStep: steps.length - 1,
      start: legStart,
      end: t,
      durationNs: leg.duration_ns,
    });
  });

  const timeline = { steps, legs, segments, total: t, stepStart, stepEnd };
  return t > dur.cap ? scale(timeline, dur.cap / t) : timeline;
}

function scale(timeline: Timeline, k: number): Timeline {
  return {
    ...timeline,
    total: timeline.total * k,
    stepStart: timeline.stepStart.map((v) => v * k),
    stepEnd: timeline.stepEnd.map((v) => v * k),
    legs: timeline.legs.map((leg) => ({ ...leg, start: leg.start * k, end: leg.end * k })),
    segments: timeline.segments.map((segment) => ({ ...segment, start: segment.start * k, end: segment.end * k })),
  };
}

/** The segment showing at time `t`. Past the end, the last one. */
export function segmentAt(timeline: Timeline, t: number): Segment | undefined {
  const { segments } = timeline;
  let low = 0;
  let high = segments.length - 1;
  while (low < high) {
    const mid = (low + high) >> 1;
    const segment = segments[mid];
    if (segment && t >= segment.end) low = mid + 1;
    else high = mid;
  }
  return segments[low];
}

/** The step the learner is looking at: the last one whose dwell has begun. -1 before the first. */
export function stepAt(timeline: Timeline, t: number): number {
  let found = -1;
  for (let i = 0; i < timeline.stepStart.length; i++) {
    if ((timeline.stepStart[i] ?? Infinity) <= t) found = i;
    else break;
  }
  return found;
}

/** Where a paused player rests to show step `i` complete. */
export function restOf(timeline: Timeline, i: number): number {
  const end = timeline.stepEnd[i];
  return end === undefined ? 0 : Math.max(timeline.stepStart[i] ?? 0, end - 1);
}
