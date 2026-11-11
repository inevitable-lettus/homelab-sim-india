// Board geometry. Node cards have a fixed size so cables and packet positions
// can be worked out from node positions alone.
import type { Medium } from '../lib/types';

export const NODE_W = 160;
export const NODE_H = 64;
const GAP = 5;

export interface Point {
  x: number;
  y: number;
}

/** Where a line from the centre of one card towards another leaves the card. */
function exit(centre: Point, toward: Point): Point {
  const dx = toward.x - centre.x;
  const dy = toward.y - centre.y;
  if (dx === 0 && dy === 0) return centre;
  const scale = Math.min(
    (NODE_W / 2 + GAP) / Math.max(Math.abs(dx), 1e-6),
    (NODE_H / 2 + GAP) / Math.max(Math.abs(dy), 1e-6),
  );
  return { x: centre.x + dx * scale, y: centre.y + dy * scale };
}

const wired = (medium: Medium) => medium !== 'cellular' && medium !== 'wifi';

/**
 * The cable between two cards. A wired cable hangs with a little slack, and
 * more when the cards are closer than they were laid out. A radio link is
 * straight.
 */
export function cablePath(a: Point, b: Point, medium: Medium, rest: number): { d: string; mid: Point } {
  const start = exit(a, b);
  const end = exit(b, a);
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const length = Math.hypot(dx, dy);
  const mid = { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 };
  if (!wired(medium) || length < 1) {
    return { d: `M${start.x} ${start.y}L${end.x} ${end.y}`, mid };
  }
  const slack = Math.min(54, 7 + Math.max(0, rest - length) * 0.5);
  // Gravity pulls a mostly level cable down. A mostly upright one bows aside.
  const level = Math.abs(dx) >= Math.abs(dy);
  const control = level ? { x: mid.x, y: mid.y + slack * 2 } : { x: mid.x + slack * 1.4, y: mid.y };
  const label = level ? { x: mid.x, y: mid.y + slack } : { x: mid.x + slack * 0.7, y: mid.y };
  return { d: `M${start.x} ${start.y}Q${control.x} ${control.y} ${end.x} ${end.y}`, mid: label };
}

/** The laid-out distance between two cards' cable ends, used as the cable's length. */
export function restLength(a: Point, b: Point): number {
  const start = exit(a, b);
  const end = exit(b, a);
  return Math.hypot(end.x - start.x, end.y - start.y);
}

export const centreOf = (position: Point): Point => ({ x: position.x + NODE_W / 2, y: position.y + NODE_H / 2 });
