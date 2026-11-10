// Phase 1 spike, throwaway. Minimal buildTimeline: pure, no DOM.
// A trace step says what happened AT a node. Travel between two different
// nodes becomes a hop segment in front of the step's own segment.

export function buildTimeline(steps, edges, dur) {
  const segments = [];
  let t = 0;
  const push = (seg, length) => {
    segments.push({ ...seg, start: t, end: t + length });
    t += length;
  };
  const edgeBetween = (a, b) => {
    for (const e of edges) {
      if (e.source === a && e.target === b) return { edgeId: e.id, reverse: false };
      if (e.source === b && e.target === a) return { edgeId: e.id, reverse: true };
    }
    return null;
  };

  let anchor = null; // where the packet rests: an edge end
  steps.forEach((step, i) => {
    const prev = steps[i - 1];
    if (prev && prev.at !== step.at) {
      const hop = edgeBetween(prev.at, step.at);
      if (hop) {
        push({ kind: 'hop', step: i, ...hop, packet: prev.packet }, dur.hop);
        anchor = { edgeId: hop.edgeId, atEnd: !hop.reverse };
      }
    }
    if (!anchor) {
      const next = steps[i + 1];
      const hop = next && edgeBetween(step.at, next.at);
      if (hop) anchor = { edgeId: hop.edgeId, atEnd: hop.reverse };
    }
    const changes = step.action === 'rewrite' || step.action === 'encapsulate';
    const length = changes ? dur.rewrite : step.action === 'drop' ? dur.drop : dur.pause;
    push(
      {
        kind: step.action,
        step: i,
        at: step.at,
        anchor,
        before: changes && prev ? prev.packet : step.packet,
        packet: step.packet,
        reason: step.reason,
      },
      length,
    );
  });
  return { segments, total: t, stepEnds: steps.map((_, i) => segments.findLast((s) => s.step === i).end) };
}

// cubic-bezier(x1, y1, x2, y2) as a function, so playback uses the same curve
// as the CSS token.
export function bezier(css) {
  const [x1, y1, x2, y2] = css.match(/-?[\d.]+/g).map(Number);
  const cx = 3 * x1;
  const bx = 3 * (x2 - x1) - cx;
  const ax = 1 - cx - bx;
  const cy = 3 * y1;
  const by = 3 * (y2 - y1) - cy;
  const ay = 1 - cy - by;
  return (x) => {
    let u = x;
    for (let i = 0; i < 8; i++) {
      const fx = ((ax * u + bx) * u + cx) * u - x;
      const d = (3 * ax * u + 2 * bx) * u + cx;
      if (Math.abs(fx) < 1e-5 || d === 0) break;
      u -= fx / d;
    }
    return ((ay * u + by) * u + cy) * u;
  };
}
