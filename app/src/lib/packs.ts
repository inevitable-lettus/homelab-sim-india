// Mirror of the schema v4 pack types in crates/sim-core/src/packs.rs.
// The app shows these values. It never judges with them: the core does.
import type { Id, Proto, World } from './types';

export type Basis = 'official' | 'review' | 'community';

/** A plain value, a value with its source, or the word `unknown`. */
export type Cap<T> = T | 'unknown' | { value: T; source: string; retrieved_at: string; basis?: Basis; note?: string };

export function capValue<T>(cap: Cap<T> | undefined): T | undefined {
  if (cap === undefined || cap === 'unknown') return undefined;
  if (typeof cap === 'object' && cap !== null && 'value' in cap && 'source' in cap) return cap.value;
  return cap as T;
}

export function capSource<T>(cap: Cap<T> | undefined): { source: string; retrieved_at: string; basis: Basis; note?: string } | undefined {
  if (typeof cap === 'object' && cap !== null && 'value' in cap && 'source' in cap) {
    return { source: cap.source, retrieved_at: cap.retrieved_at, basis: cap.basis ?? 'official', note: cap.note };
  }
  return undefined;
}

export type PackKind = 'archetype' | 'device' | 'learner';
export type Category =
  | 'computer'
  | 'router'
  | 'switch'
  | 'access_point'
  | 'ups'
  | 'inverter'
  | 'dc_ups'
  | 'drive'
  | 'enclosure'
  | 'usb_nic'
  | 'usb_hub'
  | 'client';
export type Form = 'laptop' | 'desktop' | 'mini_pc' | 'sbc' | 'phone' | 'tablet' | 'nas' | 'rack' | 'other';
export type Availability = 'sold_new' | 'used_common' | 'import_only' | 'discontinued';
export type Arch = 'x86_64' | 'aarch64' | 'armv7' | 'armv6' | 'riscv64' | 'ppc64le' | 's390x';
export type Firmware = 'uefi' | 'bios' | 'uefi_and_bios' | 'apple_silicon' | 'uboot' | 'android_bootloader';
export type AccelApi = 'vaapi' | 'qsv' | 'nvenc' | 'amf' | 'videotoolbox' | 'v4l2m2m' | 'rkmpp';
export type Codec = 'h264' | 'hevc_8' | 'hevc_10' | 'av1' | 'vp9';
export type WifiGen = 'none' | 'wifi4' | 'wifi5' | 'wifi6' | 'wifi6e' | 'wifi7';
export type UsbKind = 'usb2_a' | 'usb5_a' | 'usb10_a' | 'usb5_c' | 'usb10_c' | 'usb40_c';
export type BayKind = 'sata_2_5' | 'sata_3_5' | 'm2_nvme' | 'm2_sata' | 'msata' | 'sd_card';
export type Support = 'full' | 'partial' | 'none';
export type OsFamily = 'linux' | 'macos' | 'windows' | 'android' | 'bsd';
export type InstallMethod = 'native' | 'container' | 'lxc' | 'addon' | 'package';
export type ServiceCategory =
  | 'media'
  | 'photos'
  | 'files'
  | 'dns'
  | 'automation'
  | 'passwords'
  | 'sync'
  | 'backup'
  | 'monitoring'
  | 'proxy'
  | 'vpn'
  | 'tunnel'
  | 'games'
  | 'downloads'
  | 'dev';
export type Provides =
  | 'dns_server'
  | 'dhcp_server'
  | 'media_server'
  | 'photo_backup'
  | 'file_share'
  | 'vpn_endpoint'
  | 'tunnel_connector'
  | 'reverse_proxy'
  | 'mesh_vpn';
export type PowerInput = { kind: 'ac_230' } | { kind: 'dc'; volts_mv: number; max_ma: number } | { kind: 'usb_c_pd'; watts: number } | { kind: 'usb_5v'; max_ma: number };

export interface Nic {
  speed_mbps?: Cap<number>;
}
export interface UsbPort {
  kind: UsbKind;
  count: number;
}
export interface Bay {
  interface: BayKind;
  count: number;
}
export interface VideoCaps {
  apis?: AccelApi[];
  decode?: Partial<Record<Codec, boolean>>;
  encode?: Partial<Record<Codec, boolean>>;
}
export interface Compute {
  arch?: Cap<Arch>;
  cores?: Cap<number>;
  threads?: Cap<number>;
  ram_gb?: Cap<number>;
  ram_max_gb?: Cap<number>;
  ram_soldered?: Cap<boolean>;
  virtualization?: Cap<boolean>;
  nested_virtualization?: Cap<boolean>;
  firmware?: Cap<Firmware>;
  video?: Cap<VideoCaps>;
  cooling?: Cap<'fanless' | 'fan'>;
  ac_power_recovery?: Cap<boolean>;
  wake_on_lan?: Cap<boolean>;
}
export interface NetCaps {
  nic?: Cap<Nic[]>;
  wifi?: Cap<WifiGen>;
}
export interface PowerCaps {
  idle_w?: Cap<number>;
  load_w?: Cap<number>;
  input?: Cap<PowerInput>;
}
export interface RouterCaps {
  lan?: Cap<Nic[]>;
  wan?: Cap<'gpon_fibre' | 'epon_fibre' | 'ethernet' | 'dsl' | 'cellular'>;
  admin?: Cap<'full' | 'limited' | 'isp_only'>;
  port_forwarding?: Cap<boolean>;
  upnp?: Cap<boolean>;
  dmz?: Cap<boolean>;
  bridge_mode?: Cap<boolean>;
  lan_dns_setting?: Cap<boolean>;
  dhcp_can_disable?: Cap<boolean>;
  dhcp_reservation?: Cap<boolean>;
  ipv6?: Cap<boolean>;
  ipv6_firewall_configurable?: Cap<boolean>;
  udp_timeout_s?: Cap<number>;
  supplied_by?: Id | null;
}
export interface UpsCaps {
  topology?: Cap<'offline' | 'line_interactive' | 'online' | 'inverter' | 'dc'>;
  va?: Cap<number>;
  max_w?: Cap<number>;
  battery_wh?: Cap<number>;
  battery_external?: Cap<boolean>;
  outlets?: Cap<number>;
  dc_outputs?: Cap<{ volts_mv: number; max_ma: number; connector?: string }[]>;
  transfer_ms?: Cap<number>;
  waveform?: Cap<'sine' | 'stepped_sine' | 'square'>;
  runtime?: Cap<{ load_w: number; minutes: number }[]>;
}
export interface DriveCaps {
  interface?: Cap<BayKind>;
  medium?: Cap<'hdd' | 'ssd'>;
  capacity_gb?: Cap<number>;
  needs_12v?: Cap<boolean>;
  rated_24x7?: Cap<boolean>;
}
export interface AdapterCaps {
  host_side?: Cap<UsbKind>;
  provides?: { nic?: Cap<Nic[]>; bays?: Cap<Bay[]>; usb?: Cap<UsbPort[]> };
  uas?: Cap<boolean>;
  needs_power?: Cap<boolean>;
}
export interface OsSupport {
  level: Support;
  lid_closed?: Cap<Support>;
  charge_limit?: Cap<boolean>;
  gaps?: string[];
}

export interface DevicePack {
  id: Id;
  kind: PackKind;
  category: Category;
  label: string;
  aka?: string[];
  released?: Cap<number>;
  form?: Cap<Form>;
  india?: Cap<Availability>;
  compute?: Compute;
  network?: NetCaps;
  ports?: { usb?: Cap<UsbPort[]> };
  storage?: { bays?: Cap<Bay[]>; soldered_gb?: Cap<number>; boot_from_usb?: Cap<boolean> };
  power?: PowerCaps;
  battery?: { wh?: Cap<number>; firmware_charge_limit?: Cap<boolean> };
  router?: RouterCaps;
  ups?: UpsCaps;
  drive?: DriveCaps;
  adapter?: AdapterCaps;
  client?: { plays?: Partial<Record<Codec, boolean>>; apps?: Id[]; ipv6?: Cap<boolean> };
  os_support?: Record<Id, Cap<OsSupport>>;
  notes?: string[];
}

export interface Source {
  url: string;
  retrieved_at: string;
  basis?: Basis;
  note?: string;
}

export interface OsPack {
  id: Id;
  label: string;
  kind: 'os' | 'hypervisor' | 'nas_os' | 'appliance';
  teaching?: boolean;
  family: OsFamily;
  arches: Arch[];
  firmware: Firmware[];
  listed_devices_only: boolean;
  requires: { virtualization?: Cap<boolean>; ram_min_mb?: Cap<number>; storage_min_gb?: Cap<number>; wired_nic?: Cap<boolean> };
  base_ram_mb: Cap<number>;
  provides: { vms?: Cap<boolean>; lxc?: Cap<boolean>; vm_accelerated_arches?: Arch[]; vm_emulated_arches?: Arch[]; native_runtimes?: Id[] };
  sources?: Source[];
  notes?: string[];
}

export interface RuntimePack {
  id: Id;
  label: string;
  teaching?: boolean;
  os_families: OsFamily[];
  via_vm: Cap<boolean>;
  vm_overhead_mb: Cap<number>;
  host_network: Cap<'yes' | 'limited' | 'no' | boolean>;
  device_passthrough: Cap<AccelApi[]>;
  cost: Cap<'free' | 'free_personal' | 'paid'>;
  sources?: Source[];
  notes?: string[];
}

export interface DefaultPort {
  proto: Proto;
  port: number;
  purpose: string;
  optional?: boolean;
  direction?: 'inbound' | 'outbound';
}

export interface ServicePack {
  id: Id;
  label: string;
  teaching?: boolean;
  category: ServiceCategory;
  install: { method: InstallMethod; os_families: OsFamily[]; arches: Cap<Arch[]>; runtime_needed: boolean; os_min_version?: string }[];
  ram_min_mb: Cap<number>;
  ram_rec_mb: Cap<number>;
  storage_min_gb: Cap<number>;
  ports: DefaultPort[];
  provides: Provides[];
  licence: Cap<string>;
  sources?: Source[];
  notes?: string[];
}

export interface Tariff {
  id: Id;
  teaching?: boolean;
  state: string;
  discom: string;
  category: string;
  effective_from: string;
  slabs: { upto_kwh?: number; paise_per_kwh: number }[];
  fixed_paise_per_month: Cap<number>;
  duty_permille: Cap<number>;
  free_units?: { upto_kwh: number; condition: string };
  source: string;
  retrieved_at: string;
  note?: string;
}

export interface Outage {
  id: Id;
  label: string;
  kind: 'teaching' | 'real';
  duration_min: number;
  per_day: number;
  isp_side: 'up' | 'down' | 'unknown';
}

export interface MediaFormat {
  label: string;
  codec: Codec;
  kbps: Cap<number>;
  teaching?: boolean;
}

export interface Template {
  id: Id;
  title: string;
  summary: string;
  world: World;
  layout: { wide: Record<Id, [number, number]>; narrow: Record<Id, [number, number]> };
}

/** The sandbox catalog, fetched only when the sandbox opens. */
export interface Catalog {
  schema_version: number;
  devices: Record<Id, DevicePack>;
  os?: Record<Id, OsPack>;
  runtimes?: Record<Id, RuntimePack>;
  services?: Record<Id, ServicePack>;
  tariffs?: Record<Id, Tariff>;
  outages?: Record<Id, Outage>;
  media?: Record<Id, MediaFormat>;
  templates?: Record<Id, Template>;
}
