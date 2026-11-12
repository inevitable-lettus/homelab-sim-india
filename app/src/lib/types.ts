// TypeScript mirror of the JSON the Rust core reads and writes. The Rust
// types in crates/sim-core are the source of truth: `model.rs` for the world,
// `bundle.rs` for the data bundle, `trace.rs`, `facts.rs` and `grade.rs` for
// results. Fields the core omits when empty are optional here.

export type Id = string;
export type Status = 'pass' | 'fail' | 'unknown';
export type Severity = 'error' | 'warning' | 'info';
export type Provenance = 'official' | 'community' | 'teaching' | 'line_check' | 'learner';
export type Tri = 'yes' | 'no' | 'unknown';
export type Proto = 'tcp' | 'udp';
export type Depth = 'guided' | 'challenge';
/** `null` is "has none". `"unknown"` is "we can't say". */
export type Addr = string | null;

// ------------------------------------------------------------------ world

export interface Service {
  id: Id;
  label: string;
  proto: Proto;
  port: number;
  bind?: 'all' | 'loopback';
  /** Sandbox: the service pack this is. */
  app?: Id;
  install?: 'native' | 'container' | 'lxc' | 'addon' | 'package';
  autostart?: boolean;
}

export interface Price {
  paise: number;
  owned?: boolean;
}

/** A part fitted to a machine: a drive, an adapter, a hub. */
export interface Mount {
  id: Id;
  pack: Id;
  into?: Id;
  price?: Price;
}

export interface MachineSettings {
  lid_closed?: boolean;
  charge_limit?: boolean;
  ac_power_recovery?: boolean;
  always_on?: boolean;
}

export interface Peer {
  public_key?: string;
  node?: Id;
  endpoint?: { host: string; port: number };
  allowed_ips?: string[];
  keepalive_s?: number;
}

export interface Tunnel {
  id: Id;
  kind: 'wireguard' | 'overlay';
  enabled: boolean;
  network?: Id;
  address?: string;
  listen_port?: number;
  public_key?: string;
  mtu?: number;
  peers?: Peer[];
}

export interface HostNode {
  kind: 'host';
  id: Id;
  label: string;
  device?: Id;
  icon?: string;
  static_ipv4?: string;
  static_ipv6?: string;
  services?: Service[];
  tunnels?: Tunnel[];
  runs_on?: Id;
  forwarding?: boolean;
  os?: Id;
  runtime?: Id;
  guest?: 'vm' | 'lxc' | 'container';
  alloc?: { ram_mb?: number; disk_gb?: number; threads?: number };
  parts?: Mount[];
  settings?: MachineSettings;
  role?: 'server' | 'client';
  busy_permille?: number;
  price?: Price;
}

export interface PortForward {
  id: Id;
  enabled: boolean;
  proto: Proto;
  external_port: number;
  to_ip: string;
  internal_port: number;
}

export interface RouterNode {
  kind: 'router';
  id: Id;
  label: string;
  device?: Id;
  lan: string;
  port_forwards?: PortForward[];
  udp_timeout_s?: number;
  ipv6_inbound?: 'allow' | 'deny';
  price?: Price;
}

export type LineMode = 'public' | 'cgnat' | 'unknown' | 'profile';

export interface IspNode {
  kind: 'isp';
  id: Id;
  label: string;
  profile?: Id;
  ipv4: LineMode;
  ipv6: 'yes' | 'no' | 'unknown';
  public_pool?: string;
  cgnat_pool?: string;
  udp_timeout_s?: number;
}

export interface InternetNode {
  kind: 'internet';
  id: Id;
  label: string;
}

export interface SwitchNode {
  kind: 'switch';
  id: Id;
  label: string;
  device?: Id;
  price?: Price;
}

/** A UPS, an inverter or a DC backup unit. */
export interface PowerNode {
  kind: 'power';
  id: Id;
  label: string;
  device?: Id;
  price?: Price;
}

export type WorldNode = HostNode | RouterNode | IspNode | InternetNode | SwitchNode | PowerNode;

/** What the learner wants from a sandbox build. */
export type Intent =
  | { kind: 'run'; id: Id; host: Id; service: Id }
  | { kind: 'reach'; id: Id; from: Id; host: Id; service: Id; via: Target }
  | { kind: 'stream'; id: Id; client: Id; host: Id; service: Id; format: Id; streams?: number; via?: Target }
  | { kind: 'home_dns'; id: Id; host: Id; service: Id; router: Id }
  | { kind: 'store'; id: Id; host: Id; gb: number; copies?: number }
  | { kind: 'survive_outage'; id: Id; outage: Id; keep: Id[] }
  | { kind: 'yearly_cost'; id: Id; max_paise: number };

export interface HomePlan {
  tariff?: Id;
  rate_paise_per_kwh?: number;
  base_kwh_month?: number;
  free_units_apply?: boolean;
  outage?: Id;
}

export type Medium = 'ethernet' | 'fibre' | 'wifi' | 'cellular' | 'backbone';

export interface Link {
  id: Id;
  a: Id;
  b: Id;
  medium: Medium;
  length_m?: number;
  rate_ab_bps?: number;
  rate_ba_bps?: number;
  mtu?: number;
  load_permille?: number;
  extra_ns?: number;
}

export type Target =
  | { kind: 'address'; ip: string }
  | { kind: 'name'; name: string }
  | { kind: 'lan_of'; node: Id }
  | { kind: 'wan_of'; node: Id }
  | { kind: 'public_of'; node: Id }
  | { kind: 'ipv6_of'; node: Id }
  | { kind: 'overlay_of'; node: Id; network: Id }
  | { kind: 'tunnel_of'; node: Id; tunnel: Id };

export interface Attempt {
  id: Id;
  label: string;
  from: Id;
  to: Target;
  proto: Proto;
  port: number;
  idle_s: number;
  packet_bytes: number;
  transfer?: { bytes: number; direction: 'request' | 'reply' };
}

export interface World {
  nodes: WorldNode[];
  links: Link[];
  overlays?: { id: Id; label: string; relay: Id; range: string; port: number }[];
  attempts: Attempt[];
  answers?: { question: Id; value: string }[];
  line_checks?: { isp: Id; result: 'public' | 'cgnat' | 'unknown' }[];
  intents?: Intent[];
  power_feeds?: { from: Id; to: Id }[];
  home?: HomePlan;
  /** Devices the learner described themselves. Typed in `packs.ts`. */
  custom_devices?: unknown[];
}

// ----------------------------------------------------------------- bundle

export interface Explanation {
  title: string;
  line: string;
  what_happened?: string;
  concept?: string;
  why_it_matters?: string;
  how_to_fix?: string;
  real_line?: string;
  go_deeper?: string;
}

export interface DialOption {
  label: string;
  target: Target;
}

export type Control =
  | { kind: 'dial'; attempt: Id; options: DialOption[] }
  | { kind: 'port_forwards'; router: Id }
  | { kind: 'overlay_join'; network: Id; nodes: Id[] }
  | { kind: 'question'; question: Id }
  | { kind: 'tunnel'; node: Id; tunnel: Id };

export interface Question {
  id: Id;
  prompt: string;
  options: { value: string; label: string }[];
}

export type GoalCheck =
  | { kind: 'delivered'; attempt: Id }
  | { kind: 'reaches'; attempt: Id; node: Id }
  | { kind: 'answer'; question: Id }
  | { kind: 'rule_passes'; rule: Id };

export interface Goal {
  id: Id;
  text: string;
  depth?: Depth;
  /** Read by the UI only to know which attempt a goal waits for. The core does the checking. */
  check: GoalCheck;
}

export interface Track {
  id: Id;
  title: string;
  summary: string;
  missions: Id[];
  planned: { title: string; concept: string }[];
}

export interface ArtifactSpec {
  id: Id;
  path: string;
  kind: 'wireguard_conf' | 'wireguard_public_key';
  label: string;
  node: Id;
  tunnel: Id;
  register_on?: Id;
  /** Starting text at guided depth. Challenge starts with an empty file. */
  starter: string;
}

export interface DepthText {
  brief: string;
  hints: string[];
}

export type Position = [number, number];

export interface Mission {
  id: Id;
  order: number;
  title: string;
  concept: string;
  scenario: string;
  guided: DepthText;
  challenge: DepthText;
  layout: { wide: Record<Id, Position>; narrow: Record<Id, Position> };
  controls: Control[];
  questions: Question[];
  goals: Goal[];
  artifacts?: ArtifactSpec[];
  debrief: { did: string; concept: Id; real_line: string };
  world: World;
}

export interface IspProfile {
  id: Id;
  label: string;
  kind: 'teaching' | 'real';
  note?: string;
}

export interface Bundle {
  schema_version: number;
  isps: Record<Id, IspProfile>;
  explanations: Record<Id, Explanation>;
  missions: Record<Id, Mission>;
  tracks: Track[];
}

// ---------------------------------------------------------------- results

export interface PacketView {
  kind: 'data' | 'handshake';
  proto: Proto;
  src: string;
  dst: string;
  bytes: number;
  reply: boolean;
  inner?: { tunnel: Id; proto: Proto; src: string; dst: string };
}

export interface HopTiming {
  bytes: number;
  mtu: number;
  rate_bps: number | null;
  load_permille: number;
  propagation_ns: number | null;
  serialization_ns: number | null;
  queue_ns: number | null;
  extra_ns: number;
  processing_ns: number;
  total_ns: number | null;
  arrive_ns: number | null;
}

export type TableKind = 'mapping_created' | 'mapping_hit' | 'forward_hit' | 'no_match' | 'expired';

export interface TableRow {
  proto: Proto;
  inside: string;
  outside: string;
  remote: string;
  timeout_s: number | null;
  idle_s: number;
}

export type StepAction =
  | 'send'
  | 'forward'
  | 'rewrite'
  | 'encapsulate'
  | 'decapsulate'
  | 'drop'
  | 'deliver'
  | 'unknown';

export interface Step {
  at: Id;
  action: StepAction;
  packet: PacketView;
  before?: PacketView;
  reason: Id;
  via_link?: Id;
  timing?: HopTiming;
  table?: { kind: TableKind; row: TableRow | null };
}

export type LegRole = 'setup' | 'request' | 'reply';

export interface Leg {
  id: string;
  role: LegRole;
  from: Id;
  steps: Step[];
  duration_ns: number | null;
}

export interface TransferEstimate {
  bytes: number;
  direction: 'request' | 'reply';
  wire_bytes: number | null;
  bottleneck_link: Id | null;
  bottleneck_bps: number | null;
  duration_ms: number | null;
}

export interface Trace {
  attempt: Id;
  status: Status;
  outcome: 'delivered' | 'dropped' | 'unknown';
  ended_at: Id | null;
  reason: Id | null;
  target: string | null;
  assumed?: { isp: Id; ipv4: 'public' | 'cgnat' | 'unknown' }[];
  legs: Leg[];
  summary: {
    hops: number;
    one_way_ns: number | null;
    round_trip_ns: number | null;
    path_mtu: number | null;
    transfer: TransferEstimate | null;
  };
}

export interface NodeFacts {
  id: Id;
  kind: WorldNode['kind'];
  ipv4: Addr;
  wan_ipv4: Addr;
  public_ipv4: Addr;
  ipv6: Addr;
  behind_carrier_nat: Tri;
  nat_layers: number;
  line?: { ipv4: 'public' | 'cgnat' | 'unknown'; provenance: Provenance; shared_ipv4: Addr };
  tunnels?: { id: Id; enabled: boolean; address: Addr }[];
}

export interface Finding {
  rule_id: Id;
  status: Status;
  severity: Severity;
  involved: Id[];
  explanation_id: Id;
  provenance: Provenance;
  debrief_only: boolean;
  /** Sandbox: the goals this finding counts towards. */
  intents?: Id[];
}

export interface Report {
  schema_version: number;
  facts: { nodes: NodeFacts[] };
  findings: Finding[];
  attempts: Trace[];
  /** Sandbox builds only. */
  build?: BuildReport;
}

export interface Budget {
  used: number | null;
  recommended?: number;
  total: number | null;
  terms?: { label: string; value: number | null }[];
}

export interface BuildReport {
  intents: { id: Id; status: Status; findings: number[]; attempt?: Id }[];
  machines: { node: Id; ram: Budget; bays: Budget; usb: Budget; ports: Budget; avg_w: number | null; battery_min?: number }[];
  power: {
    lab_avg_w: number | null;
    lab_kwh_year: number | null;
    yearly_paise: number | null;
    rate_used: { tariff: Id | null; from_paise_per_kwh: number; to_paise_per_kwh: number } | null;
    assumptions: { reason: Id; value: number }[];
  };
  ups: { node: Id; fed: Id[]; fed_avg_w: number | null; runtime_min: number | null; overloaded: boolean | null }[];
  parts: { node: Id; mount?: Id; pack: Id; label: string; price_paise: number | null; owned: boolean }[];
  price_total_paise: number | null;
  unknowns: { pack: Id; field: string; used_by: Id[] }[];
}

export interface GoalResult {
  id: Id;
  status: Status;
  reason: Id;
}

export interface ArtifactResult {
  id: Id;
  status: Status;
  reason: Id;
  problems: { line: number | null; message: string }[];
}

export interface Grade {
  mission: Id;
  depth: Depth;
  status: Status;
  goals: GoalResult[];
  artifacts: ArtifactResult[];
  report: Report;
}

export interface Resolved {
  address: Addr;
  reason: Id | null;
}

export interface Artifacts {
  files: Record<string, string>;
}
