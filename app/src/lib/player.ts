// Owns the playback clock. It knows nothing about the DOM: a renderer
// subscribes to frames, and React subscribes to the rarer step and state
// changes, so no React state changes per frame.
import { restOf, stepAt, type Timeline } from './timeline';

export interface PlayerState {
  playing: boolean;
  /** The step in view, or -1 before anything has happened. */
  step: number;
  /** True once the last step has been shown. */
  ended: boolean;
  speed: number;
}

type FrameListener = (time: number) => void;
type StateListener = (state: PlayerState) => void;

export class TimelinePlayer {
  private time = 0;
  private last = 0;
  private raf = 0;
  private state: PlayerState = { playing: false, step: -1, ended: false, speed: 1 };
  private frames = new Set<FrameListener>();
  private listeners = new Set<StateListener>();

  constructor(private timeline: Timeline) {}

  getState = () => this.state;
  getTime = () => this.time;
  getTimeline = () => this.timeline;

  onFrame(listener: FrameListener) {
    this.frames.add(listener);
    listener(this.time);
    return () => {
      this.frames.delete(listener);
    };
  }

  subscribe = (listener: StateListener) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  play() {
    if (this.timeline.total === 0) return this.toEnd();
    if (this.time >= this.timeline.total) this.time = 0;
    if (this.state.playing) return;
    this.set({ playing: true });
    document.addEventListener('visibilitychange', this.onVisibility);
    this.last = performance.now();
    this.raf = requestAnimationFrame(this.tick);
  }

  pause() {
    cancelAnimationFrame(this.raf);
    document.removeEventListener('visibilitychange', this.onVisibility);
    if (this.state.playing) this.set({ playing: false });
  }

  toggle() {
    if (this.state.playing) this.pause();
    else this.play();
  }

  seek(time: number) {
    this.pause();
    this.move(Math.min(this.timeline.total, Math.max(0, time)));
  }

  /** Jump to a step and show it complete. */
  showStep(index: number) {
    const count = this.timeline.steps.length;
    if (count === 0) return this.toEnd();
    const clamped = Math.min(count - 1, Math.max(0, index));
    this.pause();
    this.move(restOf(this.timeline, clamped), clamped === count - 1);
  }

  stepForward() {
    this.showStep(this.state.step + 1);
  }

  stepBack() {
    if (this.state.step <= 0) return this.reset();
    this.showStep(this.state.step - 1);
  }

  toEnd() {
    this.pause();
    this.move(this.timeline.total, true);
  }

  reset() {
    this.pause();
    this.time = 0;
    this.set({ step: -1, ended: false });
    this.emitFrame();
  }

  setSpeed(speed: number) {
    this.set({ speed });
  }

  dispose() {
    cancelAnimationFrame(this.raf);
    document.removeEventListener('visibilitychange', this.onVisibility);
    this.frames.clear();
    this.listeners.clear();
  }

  private move(time: number, ended = time >= this.timeline.total) {
    this.time = time;
    this.set({ step: stepAt(this.timeline, time), ended });
    this.emitFrame();
  }

  private tick = (now: number) => {
    // A long gap means the tab was asleep. Do not jump ahead to make up for it.
    const elapsed = Math.min(now - this.last, 100);
    this.last = now;
    const time = Math.min(this.timeline.total, this.time + elapsed * this.state.speed);
    const ended = time >= this.timeline.total;
    this.time = time;
    this.set({ step: stepAt(this.timeline, time), ended, playing: !ended });
    this.emitFrame();
    if (ended) document.removeEventListener('visibilitychange', this.onVisibility);
    else this.raf = requestAnimationFrame(this.tick);
  };

  private onVisibility = () => {
    if (document.hidden) this.pause();
  };

  private emitFrame() {
    this.frames.forEach((listener) => listener(this.time));
  }

  private set(change: Partial<PlayerState>) {
    const next = { ...this.state, ...change };
    const same = (Object.keys(next) as (keyof PlayerState)[]).every((key) => next[key] === this.state[key]);
    if (same) return;
    this.state = next;
    this.listeners.forEach((listener) => listener(next));
  }
}
