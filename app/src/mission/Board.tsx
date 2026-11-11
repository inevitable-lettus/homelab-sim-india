// The board: device cards, cables, and the packet that replays a trace.
// Nothing here decides anything. It draws the world and the core's facts, and
// moves a packet along the timeline the player hands it.
import {
  Handle,
  Position,
  ReactFlow,
  ReactFlowProvider,
  ViewportPortal,
  useNodesState,
  useReactFlow,
  useStore,
  type Edge,
  type EdgeProps,
  type FitViewOptions,
  type Node,
  type NodeProps,
} from '@xyflow/react';
import '@xyflow/react/dist/base.css';
import { createPortal } from 'react-dom';
import { memo, useCallback, useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { NodeIcon } from '../components/icons';
import { bezier, easing } from '../lib/motion';
import type { TimelinePlayer } from '../lib/player';
import { segmentAt, type Port, type Timeline } from '../lib/timeline';
import type { Id, Link, Medium, NodeFacts, PacketView, Position as LayoutPosition, Step, World, WorldNode } from '../lib/types';
import { NODE_H, NODE_W, cablePath, centreOf, restLength, type Point } from './geometry';
import { TableCallout } from './TableCallout';

export type LayoutMode = 'wide' | 'narrow';

interface DeviceData extends Record<string, unknown> {
  node: WorldNode;
  line: string;
  tag?: string;
  editable: boolean;
  onOpen?: (id: Id) => void;
}
type DeviceNode = Node<DeviceData, 'device'>;

interface CableData extends Record<string, unknown> {
  medium: Medium;
  rest: number;
  domId: string;
}
type CableEdge = Edge<CableData, 'cable'>;

function addressLine(node: WorldNode, facts: NodeFacts | undefined): string {
  if (!facts) return '';
  if (node.kind === 'router') return facts.wan_ipv4 ? `WAN ${facts.wan_ipv4}` : 'No WAN address';
  if (node.kind === 'isp') {
    if (facts.line?.ipv4 === 'public') return 'Public addresses';
    if (facts.line?.ipv4 === 'cgnat') return 'Carrier NAT';
    return 'Line type unknown';
  }
  if (node.kind === 'internet') return '';
  return facts.ipv4 ?? 'No address';
}

const Device = memo(function Device({ data, selected }: NodeProps<DeviceNode>) {
  const { node, line, tag, editable, onOpen } = data;
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onOpen?.(node.id);
    }
  };
  return (
    <div
      className="device"
      data-kind={node.kind}
      data-selected={selected || undefined}
      role={onOpen ? 'button' : undefined}
      tabIndex={onOpen ? 0 : undefined}
      aria-label={`${node.label}${line ? `, ${line}` : ''}${editable ? '. You can change this.' : ''}`}
      onKeyDown={onOpen ? onKeyDown : undefined}
    >
      <Handle type="target" position={Position.Top} isConnectable={false} className="device__handle" />
      <Handle type="source" position={Position.Top} isConnectable={false} className="device__handle" />
      <span className="device__icon">
        <NodeIcon kind={node.kind === 'isp' && node.id === 'mobile' ? 'mobile' : node.kind} hint={node.kind === 'host' ? node.icon : undefined} />
      </span>
      <span className="device__text">
        <span className="device__label">{node.label}</span>
        {line && <span className="device__line">{line}</span>}
      </span>
      {tag && <span className="device__tag">{tag}</span>}
      {editable && <span className="device__edit" aria-hidden="true" />}
      <span className="device__stop" aria-hidden="true" />
    </div>
  );
});

const Cable = memo(function Cable({ sourceX, sourceY, targetX, targetY, data }: EdgeProps<CableEdge>) {
  if (!data) return null;
  const { d } = cablePath({ x: sourceX, y: sourceY }, { x: targetX, y: targetY }, data.medium, data.rest);
  return (
    <g className="cable" data-medium={data.medium}>
      <path className="cable__glow" d={d} />
      <path id={data.domId} className="cable__line" d={d} />
    </g>
  );
});

const nodeTypes = { device: Device };
const edgeTypes = { cable: Cable };

// ---------------------------------------------------------------- packet

type Tone = 'data' | 'pass' | 'fail' | 'unknown';

interface ChipView {
  key: string;
  packet: PacketView;
  before?: PacketView;
  tone: Tone;
  queued: boolean;
}

function toneOf(step: Step): Tone {
  if (step.action === 'drop') return 'fail';
  if (step.action === 'unknown') return 'unknown';
  if (step.action === 'deliver') return 'pass';
  return 'data';
}

function Row({ label, value, old }: { label: string; value: string; old?: string }) {
  const changed = old !== undefined && old !== value;
  return (
    <span className="chip__row">
      <span className="chip__label">{label}</span>
      <span className="chip__values">
        {changed && <s className="chip__old">{old}</s>}
        <span className={changed ? 'chip__new' : undefined} key={value}>
          {value}
        </span>
      </span>
    </span>
  );
}

function Chip({ view, docked }: { view: ChipView; docked?: boolean }) {
  const { packet, before } = view;
  return (
    <div className={docked ? 'chip chip--docked' : 'chip'} data-tone={view.tone} data-wrapped={packet.inner ? '' : undefined}>
      {view.queued && <span className="chip__queue">waiting in the queue</span>}
      {packet.kind === 'handshake' && <span className="chip__kind">tunnel hello</span>}
      <Row label="from" value={packet.src} old={before?.src} />
      <Row label="to" value={packet.dst} old={before?.dst} />
      {packet.inner && (
        <span className="chip__inner">
          wraps {packet.inner.src} to {packet.inner.dst}
        </span>
      )}
    </div>
  );
}

/** Half the widest label, in screen pixels. */
const CHIP_HALF = 104;

const ease = { travel: bezier(easing.inOut), settle: bezier(easing.out) };

interface StageProps {
  player: TimelinePlayer;
  links: Link[];
  prefix: string;
  refresh: number;
  showCallout: boolean;
  /** Where the label sits when it does not travel with the packet. */
  dock: HTMLElement | null;
  nodeLabel: (id: Id) => string;
  nodeKind: (id: Id) => WorldNode['kind'] | undefined;
}

/** Moves one packet along the timeline. Position is written straight to the DOM each frame. */
function PacketStage({ player, links, prefix, refresh, showCallout, dock, nodeLabel, nodeKind }: StageProps) {
  const flow = useReactFlow();
  const zoom = useStore((state) => state.transform[2]);
  const root = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<ChipView | null>(null);
  const [tableStep, setTableStep] = useState<{ step: Step; at: Point } | null>(null);

  useEffect(() => {
    const el = root.current;
    const board = el?.closest<HTMLElement>('.react-flow');
    if (!el || !board) return;
    const timeline: Timeline = player.getTimeline();
    const linkById = new Map(links.map((link) => [link.id, link]));
    const marked: Element[] = [];
    let key = '';
    let shift = 0;

    const pathOf = (link: Id) => document.getElementById(prefix + link) as SVGPathElement | null;
    const centre = (node: Id): Point => centreOf(flow.getNode(node)?.position ?? { x: 0, y: 0 });
    const portPoint = (port: Port): Point => {
      const path = port.link ? pathOf(port.link) : null;
      const link = port.link ? linkById.get(port.link) : undefined;
      if (!path || !link) return centre(port.node);
      return path.getPointAtLength(link.a === port.node ? 0 : path.getTotalLength());
    };
    const mark = (target: Element | null | undefined, value: string) => {
      if (!target) return;
      target.setAttribute('data-live', value);
      marked.push(target);
    };
    const clearMarks = () => {
      while (marked.length) marked.pop()?.removeAttribute('data-live');
    };
    const nodeEl = (id: Id) => board.querySelector(`.react-flow__node[data-id="${CSS.escape(id)}"]`);

    const draw = (time: number) => {
      const state = player.getState();
      const segment = state.step < 0 && !state.playing ? undefined : segmentAt(timeline, time);
      if (!segment || segment.kind === 'gap') {
        el.style.opacity = '0';
        if (key !== 'hidden') {
          key = 'hidden';
          clearMarks();
          setView(null);
          setTableStep(null);
        }
        return;
      }
      const span = segment.end - segment.start;
      const local = span > 0 ? Math.min(1, Math.max(0, (time - segment.start) / span)) : 1;
      const flat = timeline.steps[segment.step];
      if (!flat) return;

      let point: Point;
      let phase = 'travel';
      let queued = false;
      if (segment.kind === 'hop') {
        const path = pathOf(segment.link);
        if (!path) return;
        // Time spent waiting for the link is shown as waiting, at the near end.
        const wait = segment.queueShare > 0.08 ? segment.queueShare : 0;
        queued = wait > 0 && local < wait;
        const moved = ease.travel(wait > 0 ? Math.max(0, (local - wait) / (1 - wait)) : local);
        const length = path.getTotalLength();
        point = path.getPointAtLength(segment.reverse ? length * (1 - moved) : length * moved);
      } else {
        // A packet that starts or ends here waits at the card's edge, not on its label.
        const from = portPoint(segment.from.link ? segment.from : segment.to);
        const to = portPoint(segment.to.link ? segment.to : segment.from);
        const moved = ease.settle(local);
        point = { x: from.x + (to.x - from.x) * moved, y: from.y + (to.y - from.y) * moved };
        phase = flat.step.before && local < 0.32 ? 'before' : 'after';
      }

      const viewport = flow.getViewport();
      el.style.opacity = '1';
      el.style.transform = `translate(${point.x}px, ${point.y}px) scale(${1 / (viewport.zoom || 1)})`;
      // Keep the label inside the board near its left and right edges.
      const screenX = point.x * viewport.zoom + viewport.x;
      const room = CHIP_HALF + 6;
      const nudge = Math.round(Math.max(room - screenX, Math.min(0, board.clientWidth - room - screenX)));
      if (nudge !== shift) {
        shift = nudge;
        el.style.setProperty('--shift', `${nudge}px`);
      }

      const next = `${segment.kind}:${segment.step}:${phase}:${queued}`;
      if (next === key) return;
      key = next;
      clearMarks();
      if (segment.kind === 'hop') {
        mark(pathOf(segment.link)?.parentElement, 'on');
        setView({ key: next, packet: flat.previous?.packet ?? flat.step.packet, tone: 'data', queued });
        setTableStep(null);
        return;
      }
      const stopped = flat.step.action === 'drop' || flat.step.action === 'unknown';
      mark(nodeEl(flat.step.at), stopped ? flat.step.action : flat.step.action === 'deliver' ? 'deliver' : 'on');
      if (phase === 'before') {
        setView({ key: next, packet: flat.step.before ?? flat.step.packet, tone: 'data', queued: false });
      } else {
        setView({ key: next, packet: flat.step.packet, before: flat.step.before, tone: toneOf(flat.step), queued: false });
      }
      const position = flow.getNode(flat.step.at)?.position;
      setTableStep(flat.step.table && position ? { step: flat.step, at: position } : null);
    };

    const stop = player.onFrame(draw);
    return () => {
      stop();
      clearMarks();
    };
  }, [player, links, prefix, flow, refresh]);

  return (
    <>
      <div ref={root} className="packet" style={{ opacity: 0 }} aria-hidden="true">
        <span className="packet__pip" data-tone={view?.tone} />
        {view && !dock && <span className="packet__stem" />}
        {view && !dock && <Chip view={view} />}
      </div>
      {view && dock && createPortal(<Chip view={view} docked />, dock)}
      {showCallout && tableStep && (
        <div
          className="callout-anchor"
          style={{
            transform: `translate(${tableStep.at.x + NODE_W / 2}px, ${tableStep.at.y + NODE_H + 10}px) scale(${1 / (zoom || 1)})`,
          }}
          aria-hidden="true"
        >
          <TableCallout step={tableStep.step} nodeLabel={nodeLabel} nodeKind={nodeKind} />
        </div>
      )}
    </>
  );
}

// ------------------------------------------------------------------ board

export interface BoardProps {
  world: World;
  facts: NodeFacts[];
  positions: Record<Id, LayoutPosition>;
  mode: LayoutMode;
  player: TimelinePlayer | null;
  /** Nodes the learner can change in this mission. */
  editable?: ReadonlySet<Id>;
  selected?: Id | null;
  onOpen?: (id: Id) => void;
  /** Width on the right that a panel covers. The selected card is kept clear of it. */
  covered?: number;
  /** A board that only plays, with nothing to tap. */
  passive?: boolean;
  label: string;
}

function BoardInner({ world, facts, positions, mode, player, editable, selected, onOpen, covered = 0, passive, label }: BoardProps) {
  const flow = useReactFlow();
  const prefix = `cable-${useId().replace(/:/g, '')}-`;
  const [refresh, setRefresh] = useState(0);
  const factsById = useMemo(() => new Map(facts.map((f) => [f.id, f])), [facts]);

  const laidOut = useMemo(
    () =>
      world.nodes.map((node): DeviceNode => {
        const [x, y] = positions[node.id] ?? [0, 0];
        return { id: node.id, type: 'device', position: { x, y }, data: { node, line: '', editable: false } };
      }),
    [world.nodes, positions],
  );
  const [nodes, setNodes, onNodesChange] = useNodesState<DeviceNode>(laidOut);

  // New layout or new world: put the cards back where the mission placed them.
  useEffect(() => {
    setNodes(laidOut);
  }, [laidOut, setNodes]);

  const shown = useMemo(
    () =>
      nodes.map((item): DeviceNode => {
        const node = world.nodes.find((n) => n.id === item.id) ?? item.data.node;
        const nodeFacts = factsById.get(item.id);
        const mesh = nodeFacts?.tunnels?.find((t) => t.enabled && t.address);
        return {
          ...item,
          selected: selected === item.id,
          data: {
            node,
            line: addressLine(node, nodeFacts),
            tag: mesh ? 'on mesh' : undefined,
            editable: editable?.has(item.id) ?? false,
            onOpen: passive ? undefined : onOpen,
          },
        };
      }),
    [nodes, world.nodes, factsById, selected, editable, onOpen, passive],
  );

  const edges = useMemo(
    () =>
      world.links.map((link): CableEdge => {
        const a = centreOf(pointOf(positions[link.a]));
        const b = centreOf(pointOf(positions[link.b]));
        return {
          id: link.id,
          source: link.a,
          target: link.b,
          type: 'cable',
          focusable: false,
          selectable: false,
          data: {
            medium: link.medium,
            rest: restLength(a, b),
            domId: prefix + link.id,
          },
        };
      }),
    [world.links, positions, prefix],
  );

  // Room is kept above the cards for the packet's label: it rides above the
  // packet on a wide board and sits in a strip at the top of a narrow one.
  const fitOptions = useMemo(
    (): FitViewOptions => ({
      padding:
        mode === 'wide'
          ? { top: '92px', right: '24px', bottom: '24px', left: '24px' }
          : { top: '112px', right: '14px', bottom: '14px', left: '14px' },
      maxZoom: 1.15,
      minZoom: 0.35,
    }),
    [mode],
  );
  const fit = useCallback(() => {
    void flow.fitView(fitOptions);
  }, [flow, fitOptions]);
  const [dock, setDock] = useState<HTMLDivElement | null>(null);

  const frame = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = frame.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      fit();
      setRefresh((n) => n + 1);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [fit, laidOut]);

  // A panel over the right of the board must not hide the card it describes.
  useEffect(() => {
    const el = frame.current;
    if (!el) return;
    const duration = document.documentElement.dataset.motion === 'reduced' ? 0 : 240;
    const node = selected && covered ? flow.getNode(selected) : undefined;
    if (!node) {
      void flow.fitView({ ...fitOptions, duration });
      return;
    }
    const { x, y, zoom } = flow.getViewport();
    const right = (node.position.x + NODE_W) * zoom + x;
    const left = node.position.x * zoom + x;
    const limit = el.clientWidth - covered - 20;
    const shift = right > limit ? limit - right : left < 20 ? 20 - left : 0;
    if (shift !== 0) void flow.setViewport({ x: x + shift, y, zoom }, { duration });
  }, [selected, covered, flow, fitOptions]);

  const nodeLabel = useCallback((id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id, [world.nodes]);
  const nodeKind = useCallback((id: Id) => world.nodes.find((n) => n.id === id)?.kind, [world.nodes]);

  return (
    <div ref={frame} className="board" data-mode={mode} role="group" aria-label={label}>
      <ReactFlow
        nodes={shown}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        onNodesChange={onNodesChange}
        onNodeClick={passive ? undefined : (_, node) => onOpen?.(node.id)}
        onNodeDrag={() => setRefresh((n) => n + 1)}
        onMoveEnd={() => setRefresh((n) => n + 1)}
        nodesDraggable={!passive && mode === 'wide'}
        nodesConnectable={false}
        nodesFocusable={false}
        edgesFocusable={false}
        elementsSelectable={false}
        panOnDrag={!passive && mode === 'wide'}
        zoomOnScroll={false}
        zoomOnPinch={!passive && mode === 'wide'}
        zoomOnDoubleClick={false}
        preventScrolling={false}
        minZoom={0.35}
        maxZoom={1.6}
        fitView
        fitViewOptions={fitOptions}
        deleteKeyCode={null}
      >
        <ViewportPortal>
          {player && (
            <PacketStage
              player={player}
              links={world.links}
              prefix={prefix}
              refresh={refresh}
              showCallout={mode === 'wide'}
              dock={mode === 'narrow' ? dock : null}
              nodeLabel={nodeLabel}
              nodeKind={nodeKind}
            />
          )}
        </ViewportPortal>
      </ReactFlow>
      {mode === 'narrow' && <div ref={setDock} className="board__dock" aria-hidden="true" />}
    </div>
  );
}

function pointOf(position: LayoutPosition | undefined): Point {
  return { x: position?.[0] ?? 0, y: position?.[1] ?? 0 };
}

export function Board(props: BoardProps) {
  return (
    <ReactFlowProvider>
      <BoardInner {...props} />
    </ReactFlowProvider>
  );
}
