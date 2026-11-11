// Motion tokens from context-docs/Animations.md. Tune on real devices here,
// never inline. Times are milliseconds at 1x.
export const duration = {
  instant: 80,
  fast: 160,
  base: 240,
  /** Used when the core has no timing for a hop. */
  hop: 520,
  /** A hop's time is its share of the slowest hop's time, within these bounds. */
  hopMin: 170,
  hopMax: 900,
  appear: 260,
  forward: 150,
  rewrite: 560,
  pause: 280,
  drop: 900,
  deliver: 520,
  legGap: 420,
  win: 700,
  /** Animations.md: long sequences cap at about 12 seconds at 1x. */
  cap: 12000,
} as const;

export type Durations = { readonly [K in keyof typeof duration]: number };

export const easing = {
  out: 'cubic-bezier(0.22, 1, 0.36, 1)',
  inOut: 'cubic-bezier(0.65, 0, 0.35, 1)',
  in: 'cubic-bezier(0.55, 0, 1, 0.45)',
} as const;

export const playbackSpeeds = [0.5, 1, 2] as const;
export const maxPacketsOnScreen = 3;

/** `cubic-bezier(x1, y1, x2, y2)` as a function, so playback uses the same curve as the CSS token. */
export function bezier(css: string): (x: number) => number {
  const [x1 = 0, y1 = 0, x2 = 1, y2 = 1] = (css.match(/-?[\d.]+/g) ?? []).map(Number);
  const cx = 3 * x1;
  const bx = 3 * (x2 - x1) - cx;
  const ax = 1 - cx - bx;
  const cy = 3 * y1;
  const by = 3 * (y2 - y1) - cy;
  const ay = 1 - cy - by;
  return (x) => {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
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
