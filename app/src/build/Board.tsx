// The sandbox board: parts as cards, cables as lines. Positions are the
// learner's own layout and never go to the core.
import { useRef, useState, type KeyboardEvent, type PointerEvent } from 'react';
import { NodeIcon } from '../components/icons';
import { StatusMark } from '../components/Status';
import type { Budget, Id, Position, Report, Status, World, WorldNode } from '../lib/types';

export const CARD_W = 160;
export const CARD_H = 92;

export const STATUS_WORD: Record<Status, string> = { pass: 'Works in this model', fail: "Won't work", unknown: "Can't tell yet" };

/** Where every node sits: the saved place, or the next free grid slot. */
export function placed(world: World, layout: Record<Id, Position>): Record<Id, Position> {
  const out: Record<Id, Position> = {};
  let spare = 0;
  for (const node of world.nodes) {
    out[node.id] = layout[node.id] ?? [(spare % 4) * 190, 500 + Math.floor(spare++ / 4) * 130];
  }
  return out;
}

/** The worst result among the goals a node is part of. */
function nodeStatus(report: Report | null, world: World, id: Id): Status | null {
  const build = report?.build;
  if (!report || !build) return null;
  let worst: Status | null = null;
  const rank = { pass: 0, unknown: 1, fail: 2 };
  for (const result of build.intents) {
    const intent = world.intents?.find((i) => i.id === result.id);
    const direct = intent && 'host' in intent && intent.host === id;
    const named = result.findings.some((index) => {
      const finding = report.findings[index];
      return finding && finding.status !== 'pass' && finding.severity !== 'info' && finding.involved.includes(id);
    });
    if ((direct || named) && (worst === null || rank[result.status] > rank[worst])) worst = result.status;
  }
  return worst;
}

function Bar({ budget, unit }: { budget: Budget; unit: string }) {
  if (budget.total === null || budget.used === null) return <span className="bcard__ram">Memory: can't tell</span>;
  const share = Math.min(100, Math.round((budget.used / Math.max(1, budget.total)) * 100));
  return (
    <span className="bcard__ram" data-over={budget.used > budget.total}>
      <span className="bcard__bar" aria-hidden="true">
        <span style={{ width: `${share}%` }} />
      </span>
      {budget.used} of {budget.total} {unit}
    </span>
  );
}

const iconKind = (node: WorldNode) => (node.kind === 'switch' ? 'router' : node.kind === 'power' ? 'host' : node.kind);
const iconHint = (node: WorldNode) => (node.kind === 'host' && node.role === 'client' ? 'phone' : undefined);

export function Board(props: {
  world: World;
  layout: Record<Id, Position>;
  report: Report | null;
  selected: Id | null;
  connecting: Id | null;
  showPower: boolean;
  onSelect: (id: Id | null) => void;
  onMove: (id: Id, at: Position) => void;
}) {
  const at = placed(props.world, props.layout);
  const [drag, setDrag] = useState<{ id: Id; dx: number; dy: number; x: number; y: number; moved: boolean } | null>(null);
  const surface = useRef<HTMLDivElement>(null);
  const where = (id: Id): Position => (drag?.id === id ? [drag.x, drag.y] : (at[id] ?? [0, 0]));
  const centre = (id: Id): Position => {
    const [x, y] = where(id);
    return [x + CARD_W / 2, y + CARD_H / 2];
  };
  const width = Math.max(740, ...Object.values(at).map(([x]) => x + CARD_W + 40));
  const height = Math.max(480, ...Object.values(at).map(([, y]) => y + CARD_H + 40));

  const down = (event: PointerEvent<HTMLButtonElement>, id: Id) => {
    const [x, y] = where(id);
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({ id, dx: event.clientX - x, dy: event.clientY - y, x, y, moved: false });
  };
  const move = (event: PointerEvent<HTMLButtonElement>) => {
    if (!drag) return;
    const x = Math.max(0, Math.round((event.clientX - drag.dx) / 10) * 10);
    const y = Math.max(0, Math.round((event.clientY - drag.dy) / 10) * 10);
    if (x !== drag.x || y !== drag.y) setDrag({ ...drag, x, y, moved: true });
  };
  const up = () => {
    if (!drag) return;
    if (drag.moved) props.onMove(drag.id, [drag.x, drag.y]);
    else props.onSelect(drag.id);
    setDrag(null);
  };
  const key = (event: KeyboardEvent<HTMLButtonElement>, id: Id) => {
    const step: Record<string, Position> = { ArrowLeft: [-20, 0], ArrowRight: [20, 0], ArrowUp: [0, -20], ArrowDown: [0, 20] };
    const by = step[event.key];
    if (by && event.altKey) {
      event.preventDefault();
      const [x, y] = where(id);
      props.onMove(id, [Math.max(0, x + by[0]), Math.max(0, y + by[1])]);
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      props.onSelect(id);
    }
  };

  const guests = props.world.nodes.filter((node) => node.kind === 'host' && node.runs_on);
  return (
    <div className="bboard" ref={surface} data-connecting={props.connecting !== null} onPointerDown={(event) => event.target === event.currentTarget && props.onSelect(null)}>
      <div className="bboard__surface" style={{ width, height }}>
        <svg className="bboard__wires" width={width} height={height} aria-hidden="true">
          {props.world.links.map((link) => {
            const [x1, y1] = centre(link.a);
            const [x2, y2] = centre(link.b);
            return (
              <g key={link.id}>
                <line x1={x1} y1={y1} x2={x2} y2={y2} className="wire" data-medium={link.medium} />
                <text x={(x1 + x2) / 2} y={(y1 + y2) / 2 - 6} className="wire__label">
                  {link.medium === 'wifi' ? 'Wi-Fi' : link.medium}
                </text>
              </g>
            );
          })}
          {guests.map((node) => {
            if (node.kind !== 'host' || !node.runs_on) return null;
            const [x1, y1] = centre(node.id);
            const [x2, y2] = centre(node.runs_on);
            return <line key={node.id} x1={x1} y1={y1} x2={x2} y2={y2} className="wire wire--guest" />;
          })}
          {props.showPower &&
            (props.world.power_feeds ?? []).map((feed) => {
              const [x1, y1] = centre(feed.from);
              const [x2, y2] = centre(feed.to);
              return <line key={`${feed.from}>${feed.to}`} x1={x1} y1={y1 + 10} x2={x2} y2={y2 + 10} className="wire wire--power" />;
            })}
        </svg>
        {props.world.nodes.map((node) => {
          const [x, y] = where(node.id);
          const status = nodeStatus(props.report, props.world, node.id);
          const machine = props.report?.build?.machines.find((m) => m.node === node.id);
          return (
            <button
              key={node.id}
              type="button"
              className="bcard"
              data-testid={`node-${node.id}`}
              data-kind={node.kind}
              data-selected={props.selected === node.id}
              data-source={props.connecting === node.id}
              aria-pressed={props.selected === node.id}
              style={{ left: x, top: y, width: CARD_W, minHeight: CARD_H }}
              onPointerDown={(event) => down(event, node.id)}
              onPointerMove={move}
              onPointerUp={up}
              onPointerCancel={() => setDrag(null)}
              onKeyDown={(event) => key(event, node.id)}
            >
              <span className="bcard__head">
                <NodeIcon kind={iconKind(node)} hint={iconHint(node)} size={18} />
                <span className="bcard__label">{node.label}</span>
              </span>
              {node.kind === 'power' && <span className="bcard__sub">Backup power</span>}
              {node.kind === 'host' && node.runs_on && <span className="bcard__sub">{node.guest === 'vm' ? 'Virtual machine' : node.guest === 'lxc' ? 'LXC container' : 'Container'}</span>}
              {machine && <Bar budget={machine.ram} unit="MB" />}
              {status && <StatusMark status={status} word={STATUS_WORD[status]} />}
            </button>
          );
        })}
      </div>
    </div>
  );
}
