// Phase 1 spike, throwaway. A labelled packet follows React Flow edges.
// React renders the board once. Playback writes to the DOM directly.
import { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Position, ReactFlow, ReactFlowProvider, ViewportPortal, useStoreApi } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import './style.css';
import fixture from '../fixtures/cgnat-traces.json';
import { duration, easing, playbackSpeeds } from '../../app/src/lib/motion.ts';
import { bezier, buildTimeline } from './timeline.js';

const nodes = fixture.nodes.map((n) => ({
  id: n.id,
  position: { x: n.x, y: n.y },
  data: { label: n.label },
  sourcePosition: Position.Right,
  targetPosition: Position.Left,
  draggable: false,
}));
const edges = fixture.edges.map((e) => ({ ...e, selectable: false }));
const travel = bezier(easing.inOut);

class Player {
  constructor(refs, getZoom) {
    this.refs = refs;
    this.getZoom = getZoom;
    this.t = 0;
    this.speed = 1;
    this.playing = false;
    this.segment = null;
    this.paths = new Map();
    this.onState = () => {};
    this.frame = this.frame.bind(this);
    document.addEventListener('visibilitychange', () => document.hidden && this.pause());
  }
  // ViewportPortal mounts after the first effect, so resolve elements lazily.
  get el() {
    if (!this.els && this.refs.packet.current) {
      this.els = Object.fromEntries(Object.entries(this.refs).map(([k, r]) => [k, r.current]));
    }
    return this.els;
  }
  load(steps) {
    this.pause();
    this.timeline = buildTimeline(steps, fixture.edges, duration);
    this.segment = null;
    this.seek(0);
  }
  path(edgeId) {
    let p = this.paths.get(edgeId);
    if (!p) {
      const node = document.querySelector(`.react-flow__edge[data-id="${edgeId}"] .react-flow__edge-path`);
      if (!node) return null;
      p = { node, length: node.getTotalLength() };
      this.paths.set(edgeId, p);
    }
    return p;
  }
  play() {
    if (this.playing) return;
    if (this.t >= this.timeline.total) this.t = 0;
    this.playing = true;
    this.last = performance.now();
    this.el.packet.style.willChange = 'transform';
    requestAnimationFrame(this.frame);
    this.onState();
  }
  pause() {
    if (!this.playing) return;
    this.playing = false;
    if (this.el) this.el.packet.style.willChange = '';
    this.onState();
  }
  frame(now) {
    if (!this.playing) return;
    this.t = Math.min(this.timeline.total, this.t + (now - this.last) * this.speed);
    this.last = now;
    this.paint();
    if (this.t >= this.timeline.total) this.pause();
    else requestAnimationFrame(this.frame);
  }
  seek(t) {
    this.t = Math.max(0, Math.min(this.timeline.total, t));
    this.paint();
  }
  step(direction) {
    this.pause();
    const ends = this.timeline.stepEnds;
    const target =
      direction > 0
        ? (ends.find((e) => e > this.t + 0.5) ?? this.timeline.total)
        : (ends.findLast((e) => e < this.t - 0.5) ?? 0);
    this.seek(target);
  }
  // Segment changes write text and classes. Every frame writes one transform
  // and, during a rewrite, two opacities.
  paint() {
    const { segments, total } = this.timeline;
    const seg = segments.find((s) => this.t < s.end) ?? segments[segments.length - 1];
    if (seg !== this.segment) this.enter(seg);
    const p = seg.end === seg.start ? 1 : Math.min(1, (this.t - seg.start) / (seg.end - seg.start));
    const done = this.t >= total;

    let path;
    let l;
    if (seg.kind === 'hop') {
      path = this.path(seg.edgeId);
      const e = travel(p);
      l = path ? path.length * (seg.reverse ? 1 - e : e) : 0;
    } else {
      path = this.path(seg.anchor.edgeId);
      l = path && seg.anchor.atEnd ? path.length : 0;
    }
    if (!path) return;
    const point = path.node.getPointAtLength(l);
    this.debug = { path: path.node, l };
    this.el.packet.style.transform = `translate(${point.x}px, ${point.y}px) scale(${1 / this.getZoom()})`;

    const changing = seg.kind === 'rewrite' || seg.kind === 'encapsulate';
    const shown = changing ? (done ? 1 : p) : 1;
    this.el.before.style.opacity = String(1 - shown);
    this.el.after.style.opacity = String(shown);
  }
  enter(seg) {
    this.segment = seg;
    const el = this.el;
    const line = (p) => `src ${p.src}\ndst ${p.dst}`;
    el.before.textContent = line(seg.before ?? seg.packet);
    el.after.textContent = line(seg.packet);
    el.packet.dataset.kind = seg.kind;
    el.reason.textContent = seg.kind === 'hop' ? '' : `${seg.kind}${seg.reason ? `: ${seg.reason}` : ''}`;
    for (const li of el.log.children) {
      if (Number(li.dataset.step) === seg.step) li.setAttribute('aria-current', 'step');
      else li.removeAttribute('aria-current');
    }
    document.querySelector('.react-flow__node.is-active')?.classList.remove('is-active');
    if (seg.at) document.querySelector(`.react-flow__node[data-id="${seg.at}"]`)?.classList.add('is-active');
  }
}

// Measurement helpers for spikes/measure.mjs.
function recordFrames(ms) {
  return new Promise((resolve) => {
    const deltas = [];
    let last = performance.now();
    const end = last + ms;
    const tick = (now) => {
      deltas.push(now - last);
      last = now;
      if (now < end) requestAnimationFrame(tick);
      else {
        deltas.shift();
        const sorted = [...deltas].sort((a, b) => a - b);
        const sum = deltas.reduce((a, b) => a + b, 0);
        resolve({
          frames: deltas.length,
          avgFps: Number((1000 / (sum / deltas.length)).toFixed(1)),
          p95Ms: Number(sorted[Math.floor(sorted.length * 0.95)].toFixed(1)),
          maxMs: Number(sorted[sorted.length - 1].toFixed(1)),
          over34Ms: deltas.filter((d) => d > 34).length,
          over50Ms: deltas.filter((d) => d > 50).length,
        });
      }
    };
    requestAnimationFrame(tick);
  });
}
function positionError(player) {
  const { path, l } = player.debug;
  const p = path.getPointAtLength(l).matrixTransform(path.getScreenCTM());
  const r = player.el.packet.getBoundingClientRect();
  return Math.hypot(r.left - p.x, r.top - p.y);
}
function recordPositionError(player, ms) {
  return new Promise((resolve) => {
    let max = 0;
    let samples = 0;
    const end = performance.now() + ms;
    const tick = (now) => {
      max = Math.max(max, positionError(player));
      samples++;
      if (now < end) requestAnimationFrame(tick);
      else resolve({ samples, maxErrorPx: Number(max.toFixed(2)) });
    };
    requestAnimationFrame(tick);
  });
}

function Board() {
  const store = useStoreApi();
  const refs = { packet: useRef(null), before: useRef(null), after: useRef(null), reason: useRef(null), log: useRef(null) };
  const player = useRef(null);
  const [traceName, setTraceName] = useState(Object.keys(fixture.traces)[1]);
  const [ui, setUi] = useState({ playing: false, speed: 1 });
  const steps = fixture.traces[traceName];

  useEffect(() => {
    const p = new Player(refs, () => store.getState().transform[2]);
    p.onState = () => setUi({ playing: p.playing, speed: p.speed });
    player.current = p;
    window.__spike = { player: p, recordFrames, recordPositionError: (ms) => recordPositionError(p, ms) };
    // Keep the counter-scale right when the learner zooms while paused.
    return store.subscribe(() => !p.playing && p.timeline && p.paint());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  useEffect(() => {
    // Edge paths exist one frame after React Flow measures the nodes.
    let raf;
    const start = () => {
      if (document.querySelector('.react-flow__edge-path') && player.current.el) player.current.load(steps);
      else raf = requestAnimationFrame(start);
    };
    start();
    return () => cancelAnimationFrame(raf);
  }, [steps]);

  const p = () => player.current;
  return (
    <main>
      <div className="board">
        <ReactFlow nodes={nodes} edges={edges} fitView nodesConnectable={false} minZoom={0.2} proOptions={{ hideAttribution: false }}>
          <ViewportPortal>
            <div className="packet" ref={refs.packet}>
              <div className="dot" />
              <div className="label">
                <span className="addr">
                  <span ref={refs.before} className="before" />
                  <span ref={refs.after} className="after" />
                </span>
                <span ref={refs.reason} className="reason" />
              </div>
            </div>
          </ViewportPortal>
        </ReactFlow>
      </div>
      <div className="controls">
        <button onClick={() => p().step(-1)} aria-label="Step back">⏮</button>
        <button onClick={() => (ui.playing ? p().pause() : p().play())}>{ui.playing ? 'Pause' : 'Play'}</button>
        <button onClick={() => p().step(1)} aria-label="Step forward">⏭</button>
        {playbackSpeeds.map((s) => (
          <button
            key={s}
            aria-pressed={ui.speed === s}
            onClick={() => {
              p().speed = s;
              p().onState();
            }}
          >
            {s}x
          </button>
        ))}
        <select value={traceName} onChange={(e) => setTraceName(e.target.value)} aria-label="Trace">
          {Object.keys(fixture.traces).map((name) => (
            <option key={name}>{name}</option>
          ))}
        </select>
      </div>
      <ol className="log" ref={refs.log}>
        {steps.map((s, i) => (
          <li key={`${traceName}-${i}`} data-step={i}>
            <b>{s.at}</b> {s.action} · {s.packet.src} → {s.packet.dst}
            {s.reason ? <em> · {s.reason}</em> : null}
          </li>
        ))}
      </ol>
    </main>
  );
}

createRoot(document.getElementById('root')).render(
  <ReactFlowProvider>
    <Board />
  </ReactFlowProvider>,
);
