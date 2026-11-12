// Sandbox builds: keeping them, sharing them and editing them. Nothing here
// decides whether a build works. The core does that from the world.
import type { Catalog, DevicePack, Template } from './packs';
import { validateWorld, type WorldProblem } from './sim';
import type { HostNode, Id, Intent, Link, Medium, Position, World, WorldNode } from './types';

export const SCHEMA_VERSION = 4;
const STORE_KEY = 'hls.builds.v1';
/** Longest share-link payload we offer. Past this, export a file instead. */
export const SHARE_LIMIT = 8192;

export interface Layout {
  wide: Record<Id, Position>;
  narrow: Record<Id, Position>;
}

export interface SavedBuild {
  id: Id;
  name: string;
  created: string;
  updated: string;
  schema_version: number;
  world: World;
  layout: Layout;
}

// ------------------------------------------------------------ storage

let memory: SavedBuild[] | null = null;

/** False once the browser has refused storage. Builds then live in memory only. */
export let storageWorks = true;

export function listBuilds(): SavedBuild[] {
  if (memory) return memory;
  try {
    const raw = localStorage.getItem(STORE_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? (parsed as SavedBuild[]) : [];
  } catch {
    storageWorks = false;
    memory = [];
    return memory;
  }
}

function writeBuilds(builds: SavedBuild[]) {
  if (!memory) {
    try {
      localStorage.setItem(STORE_KEY, JSON.stringify(builds));
      return;
    } catch {
      storageWorks = false;
    }
  }
  memory = builds;
}

export function getBuild(id: Id): SavedBuild | undefined {
  return listBuilds().find((build) => build.id === id);
}

export function saveBuild(build: SavedBuild) {
  const next = { ...build, updated: new Date().toISOString() };
  writeBuilds([next, ...listBuilds().filter((other) => other.id !== build.id)]);
}

export function deleteBuild(id: Id) {
  writeBuilds(listBuilds().filter((build) => build.id !== id));
}

function newId(): Id {
  const taken = new Set(listBuilds().map((build) => build.id));
  for (;;) {
    const id = Math.random().toString(36).slice(2, 8);
    if (id.length === 6 && !taken.has(id)) return id;
  }
}

function wrap(name: string, world: World, layout: Layout): SavedBuild {
  const now = new Date().toISOString();
  return { id: newId(), name, created: now, updated: now, schema_version: SCHEMA_VERSION, world, layout };
}

// --------------------------------------------------------- new builds

const GIGABIT = 1_000_000_000;

/** The parts of the outside world every build has, with fixed ids. */
export const SCAFFOLD: Id[] = ['internet', 'mobile-network', 'away-phone', 'home-isp'];

export function emptyBuild(): SavedBuild {
  const world: World = {
    nodes: [
      { kind: 'internet', id: 'internet', label: 'Internet' },
      { kind: 'isp', id: 'mobile-network', label: 'Mobile network', ipv4: 'cgnat', ipv6: 'no', public_pool: '198.51.100.0/24', cgnat_pool: '100.72.5.0/24', udp_timeout_s: 30 },
      { kind: 'host', id: 'away-phone', label: 'Your phone, away from home', role: 'client', device: 'phone-client' },
      { kind: 'isp', id: 'home-isp', label: 'Home ISP', ipv4: 'unknown', ipv6: 'no', public_pool: '203.0.113.0/24', cgnat_pool: '100.64.9.0/24', udp_timeout_s: 60 },
    ],
    links: [
      { id: 'phone-mobile', a: 'away-phone', b: 'mobile-network', medium: 'cellular', length_m: 2000, rate_ab_bps: 8_000_000, rate_ba_bps: 20_000_000 },
      { id: 'mobile-internet', a: 'mobile-network', b: 'internet', medium: 'backbone', length_m: 840_000, rate_ab_bps: 10 * GIGABIT, rate_ba_bps: 10 * GIGABIT },
      { id: 'isp-internet', a: 'home-isp', b: 'internet', medium: 'backbone', length_m: 120_000, rate_ab_bps: 10 * GIGABIT, rate_ba_bps: 10 * GIGABIT },
    ],
    attempts: [],
    intents: [],
    home: {},
  };
  const wide: Record<Id, Position> = { 'away-phone': [0, 20], 'mobile-network': [190, 20], internet: [380, 20], 'home-isp': [570, 20] };
  return wrap('My build', world, { wide, narrow: {} });
}

export function fromTemplate(template: Template): SavedBuild {
  const world = structuredClone(template.world);
  world.attempts ??= [];
  world.intents ??= [];
  world.home ??= {};
  return wrap(template.title, world, structuredClone(template.layout));
}

// ------------------------------------------------------------ editing

export function uniqueId(base: string, taken: Iterable<Id>): Id {
  const used = new Set(taken);
  const stem = base.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'part';
  if (!used.has(stem)) return stem;
  for (let n = 2; ; n += 1) if (!used.has(`${stem}-${n}`)) return `${stem}-${n}`;
}

export const nodeIds = (world: World) => world.nodes.map((node) => node.id);

export function linkBetween(a: Id, b: Id, medium: Medium, taken: Iterable<Id>): Link {
  const base = { id: uniqueId(`${a}-${b}`, taken), a, b, medium };
  if (medium === 'ethernet') return { ...base, length_m: 5, rate_ab_bps: GIGABIT, rate_ba_bps: GIGABIT };
  if (medium === 'wifi') return { ...base, length_m: 6, rate_ab_bps: 100_000_000, rate_ba_bps: 100_000_000 };
  return { ...base, length_m: 8000, rate_ab_bps: 50_000_000, rate_ba_bps: 100_000_000 };
}

/** The next free place on the board for a new part. */
export function freeSlot(layout: Record<Id, Position>): Position {
  const taken = new Set(Object.values(layout).map(([x, y]) => `${x},${y}`));
  // The outside world sits along the top. The home fills in under it, from the right.
  for (let row = 0; ; row += 1) {
    for (let col = 0; col < 4; col += 1) {
      const at: Position = [570 - col * 190, 180 + row * 160];
      if (!taken.has(`${at[0]},${at[1]}`)) return at;
    }
  }
}

/** Remove a node and everything that names it. */
export function removeNode(world: World, id: Id): World {
  const gone = new Set<Id>([id]);
  // Guests go with the machine they run on.
  for (let grew = true; grew; ) {
    grew = false;
    for (const node of world.nodes) {
      if (node.kind === 'host' && node.runs_on && gone.has(node.runs_on) && !gone.has(node.id)) {
        gone.add(node.id);
        grew = true;
      }
    }
  }
  const intents = (world.intents ?? []).filter((intent) => !intentNodes(intent).some((node) => gone.has(node)));
  const kept = new Set(intents.map((intent) => intent.id));
  return {
    ...world,
    nodes: world.nodes.filter((node) => !gone.has(node.id)),
    links: world.links.filter((link) => !gone.has(link.a) && !gone.has(link.b)),
    power_feeds: (world.power_feeds ?? []).filter((feed) => !gone.has(feed.from) && !gone.has(feed.to)),
    intents: intents.map((intent) =>
      intent.kind === 'survive_outage' ? { ...intent, keep: intent.keep.filter((goal) => kept.has(goal)) } : intent,
    ),
  };
}

export function intentNodes(intent: Intent): Id[] {
  switch (intent.kind) {
    case 'run':
    case 'store':
      return [intent.host];
    case 'reach':
      return [intent.host, intent.from];
    case 'stream':
      return [intent.host, intent.client];
    case 'home_dns':
      return [intent.host, intent.router];
    case 'survive_outage':
    case 'yearly_cost':
      return [];
  }
}

export const isHost = (node: WorldNode | undefined): node is HostNode => node?.kind === 'host';

/** Every device pack a build can name: the catalog's and the learner's own. */
export function devicesOf(world: World, catalog: Catalog): Record<Id, DevicePack> {
  const own = (world.custom_devices ?? []) as DevicePack[];
  return { ...catalog.devices, ...Object.fromEntries(own.map((pack) => [pack.id, pack])) };
}

/** A goal in the learner's words. */
export function goalText(intent: Intent, world: World, catalog: Catalog): string {
  const label = (id: Id) => world.nodes.find((node) => node.id === id)?.label ?? id;
  const service = (host: Id, id: Id) => {
    const node = world.nodes.find((n) => n.id === host);
    return (isHost(node) && node.services?.find((s) => s.id === id)?.label) || id;
  };
  switch (intent.kind) {
    case 'run':
      return `Run ${service(intent.host, intent.service)} on ${label(intent.host)}`;
    case 'reach':
      return `Open ${service(intent.host, intent.service)} from ${label(intent.from)}`;
    case 'stream':
      return `Watch ${catalog.media?.[intent.format]?.label ?? intent.format} from ${service(intent.host, intent.service)} on ${label(intent.client)}`;
    case 'home_dns':
      return `Use ${service(intent.host, intent.service)} for every device in the house`;
    case 'store':
      return `Keep ${intent.gb} GB on ${label(intent.host)}${(intent.copies ?? 1) > 1 ? `, in ${intent.copies} copies` : ''}`;
    case 'survive_outage':
      return `Stay up through: ${catalog.outages?.[intent.outage]?.label ?? intent.outage}`;
    case 'yearly_cost':
      return `Keep electricity under ${rupees(intent.max_paise)} a year`;
  }
}

const inr = new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR', maximumFractionDigits: 2 });

/** Paise as rupees. Only formats a figure the core or the learner gave. */
export function rupees(paise: number): string {
  return inr.format(paise / 100);
}

// ------------------------------------------------- export, import, share

interface Envelope {
  homelab_sim_build: 1;
  schema_version: number;
  name: string;
  world: World;
  layout: Layout;
}

/** A copy with every price the learner typed removed. */
export function withoutPrices(world: World): World {
  const strip = <T extends object>(thing: T): T => {
    const copy = { ...thing } as T & { price?: unknown };
    delete copy.price;
    return copy;
  };
  return {
    ...world,
    nodes: world.nodes.map((node) =>
      node.kind === 'host' ? { ...strip(node), parts: node.parts?.map(strip) } : strip(node),
    ),
  };
}

export function toEnvelope(build: SavedBuild, includePrices: boolean): string {
  const envelope: Envelope = {
    homelab_sim_build: 1,
    schema_version: build.schema_version,
    name: build.name,
    world: includePrices ? build.world : withoutPrices(build.world),
    layout: build.layout,
  };
  return JSON.stringify(envelope);
}

export type Imported = { ok: true; build: SavedBuild } | { ok: false; problems: WorldProblem[] };

const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null;

/** Read a build someone exported or shared. The core checks it before it is loaded. */
export function fromEnvelope(text: string): Imported {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    return { ok: false, problems: [{ kind: 'parse', at: error instanceof Error ? error.message : 'not JSON' }] };
  }
  if (!isRecord(parsed) || parsed.homelab_sim_build !== 1 || !isRecord(parsed.world)) {
    return { ok: false, problems: [{ kind: 'parse', at: 'this is not a Homelab Sim build file' }] };
  }
  if (parsed.schema_version !== SCHEMA_VERSION) {
    return { ok: false, problems: [{ kind: 'parse', at: `the file is schema ${String(parsed.schema_version)} and this site reads schema ${SCHEMA_VERSION}` }] };
  }
  const problems = validateWorld(JSON.stringify(parsed.world));
  if (problems.length > 0) return { ok: false, problems };
  const world = parsed.world as unknown as World;
  world.attempts ??= [];
  world.intents ??= [];
  const layout = isRecord(parsed.layout) ? (parsed.layout as unknown as Layout) : { wide: {}, narrow: {} };
  const name = typeof parsed.name === 'string' ? parsed.name.slice(0, 80) : 'Imported build';
  return { ok: true, build: wrap(name, world, { wide: layout.wide ?? {}, narrow: layout.narrow ?? {} }) };
}

export function problemText(problem: WorldProblem): string {
  switch (problem.kind) {
    case 'parse':
      return `The file could not be read: ${problem.at}`;
    case 'duplicate-id':
      return `Two things share the id "${problem.at}".`;
    case 'unknown-node':
      return `Something points at "${problem.at}", which is not in the build.`;
    case 'unknown-pack':
      return `"${problem.at}" names a part this site has no data for.`;
    case 'unknown-service':
      return `A goal names a service that is not there: "${problem.at}".`;
    case 'loop':
      return `"${problem.at}" is plugged into itself.`;
    case 'too-big':
      return 'The build is larger than the sandbox handles.';
  }
}

async function pipe(bytes: Uint8Array<ArrayBuffer>, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const writer = stream.writable.getWriter();
  void writer.write(bytes);
  void writer.close();
  return new Uint8Array(await new Response(stream.readable).arrayBuffer());
}

/** The build as a link fragment. It stays in the address bar and is never sent anywhere. */
export async function sharePayload(build: SavedBuild, includePrices: boolean): Promise<string> {
  const packed = await pipe(new TextEncoder().encode(toEnvelope(build, includePrices)), new CompressionStream('deflate-raw'));
  let binary = '';
  for (const byte of packed) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export async function fromSharePayload(payload: string): Promise<Imported> {
  try {
    const binary = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
    const text = new TextDecoder().decode(await pipe(bytes, new DecompressionStream('deflate-raw')));
    return fromEnvelope(text);
  } catch {
    return { ok: false, problems: [{ kind: 'parse', at: 'the link is cut short or damaged' }] };
  }
}
