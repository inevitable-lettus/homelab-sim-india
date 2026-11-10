// Motion tokens from context-docs/Animations.md. Starting values: tune on real
// devices here, never inline.
export const duration = {
  instant: 80,
  fast: 160,
  base: 240,
  hop: 520,
  rewrite: 480,
  pause: 280,
  drop: 420,
  win: 700,
} as const;

export const easing = {
  out: 'cubic-bezier(0.22, 1, 0.36, 1)',
  inOut: 'cubic-bezier(0.65, 0, 0.35, 1)',
  in: 'cubic-bezier(0.55, 0, 1, 0.45)',
} as const;

export const playbackSpeeds = [0.5, 1, 2] as const;
export const maxPacketsOnScreen = 3;
