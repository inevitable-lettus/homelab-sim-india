// Pure edits to the learner's copy of a mission's world. These only record
// what the learner chose. Whether the result works is the core's call.
import type { HostNode, Id, PortForward, RouterNode, Target, World } from './types';

function edit(world: World, change: (draft: World) => void): World {
  const draft = structuredClone(world);
  change(draft);
  return draft;
}

export function setDial(world: World, attempt: Id, target: Target): World {
  return edit(world, (draft) => {
    const found = draft.attempts.find((a) => a.id === attempt);
    if (found) found.to = target;
  });
}

function router(world: World, id: Id): RouterNode | undefined {
  const node = world.nodes.find((n) => n.id === id);
  return node?.kind === 'router' ? node : undefined;
}

export function forwardsOf(world: World, id: Id): PortForward[] {
  return router(world, id)?.port_forwards ?? [];
}

export function saveForward(world: World, routerId: Id, forward: PortForward): World {
  return edit(world, (draft) => {
    const found = router(draft, routerId);
    if (!found) return;
    const list = found.port_forwards ?? [];
    const at = list.findIndex((f) => f.id === forward.id);
    if (at >= 0) list[at] = forward;
    else list.push(forward);
    found.port_forwards = list;
  });
}

export function removeForward(world: World, routerId: Id, forwardId: Id): World {
  return edit(world, (draft) => {
    const found = router(draft, routerId);
    if (found) found.port_forwards = (found.port_forwards ?? []).filter((f) => f.id !== forwardId);
  });
}

function overlayTunnel(world: World, nodeId: Id, network: Id) {
  const node = world.nodes.find((n): n is HostNode => n.id === nodeId && n.kind === 'host');
  return node?.tunnels?.find((t) => t.kind === 'overlay' && t.network === network);
}

export function isJoined(world: World, nodeId: Id, network: Id): boolean {
  return overlayTunnel(world, nodeId, network)?.enabled ?? false;
}

export function setJoined(world: World, nodeId: Id, network: Id, joined: boolean): World {
  return edit(world, (draft) => {
    const tunnel = overlayTunnel(draft, nodeId, network);
    if (tunnel) tunnel.enabled = joined;
  });
}

export function answerOf(world: World, question: Id): string | undefined {
  return world.answers?.find((a) => a.question === question)?.value;
}

export function setAnswer(world: World, question: Id, value: string): World {
  return edit(world, (draft) => {
    draft.answers = [...(draft.answers ?? []).filter((a) => a.question !== question), { question, value }];
  });
}

/** Everything a run depends on. Answers are left out: answering does not change the network. */
export function setupKey(world: World): string {
  return JSON.stringify({ ...world, answers: undefined });
}

export const sameTarget = (a: Target, b: Target) => JSON.stringify(a) === JSON.stringify(b);

/** Form check only: is this text shaped like an IPv4 address? */
export function looksLikeIpv4(text: string): boolean {
  const parts = text.trim().split('.');
  return parts.length === 4 && parts.every((p) => /^\d{1,3}$/.test(p) && Number(p) <= 255);
}

export function parsePort(text: string): number | null {
  if (!/^\d{1,5}$/.test(text.trim())) return null;
  const port = Number(text);
  return port >= 1 && port <= 65535 ? port : null;
}
