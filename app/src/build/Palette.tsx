// The parts list. It never says whether a part will work: only the report does.
import { useState } from 'react';
import { capValue, type Arch, type Catalog, type Category, type DevicePack, type WifiGen } from '../lib/packs';
import { Check, Num, Select, Text } from './fields';

const GROUPS: { id: string; label: string; categories: Category[] }[] = [
  { id: 'computers', label: 'Computers', categories: ['computer'] },
  { id: 'network', label: 'Network', categories: ['router', 'switch', 'access_point'] },
  { id: 'power', label: 'Power', categories: ['ups', 'inverter', 'dc_ups'] },
  { id: 'storage', label: 'Storage', categories: ['drive', 'enclosure'] },
  { id: 'adapters', label: 'Adapters', categories: ['usb_nic', 'usb_hub'] },
  { id: 'clients', label: 'Clients', categories: ['client'] },
];

/** A line of facts from the pack's own capabilities. */
function facts(pack: DevicePack): string {
  const out: string[] = [];
  const arch = capValue(pack.compute?.arch);
  const ram = capValue(pack.compute?.ram_gb);
  const nic = capValue(pack.network?.nic);
  const wifi = capValue(pack.network?.wifi);
  const gb = capValue(pack.drive?.capacity_gb);
  const maxW = capValue(pack.ups?.max_w);
  if (arch) out.push(arch === 'x86_64' ? 'x86' : arch === 'aarch64' ? 'ARM' : arch);
  if (ram !== undefined) out.push(`${ram} GB RAM`);
  if (nic !== undefined && pack.category === 'computer') out.push(nic.length === 0 ? 'no Ethernet port' : `${nic.length} Ethernet`);
  if (wifi && wifi !== 'none') out.push('Wi-Fi');
  if (gb !== undefined) out.push(`${gb} GB`);
  if (maxW !== undefined) out.push(`up to ${maxW} W`);
  return out.join(' · ');
}

const KIND_WORD: Record<DevicePack['kind'], string> = { archetype: 'Teaching model', device: 'Real device', learner: 'Yours' };

export function Palette(props: { catalog: Catalog; own: DevicePack[]; onAdd: (pack: DevicePack) => void; onCustom: (pack: DevicePack) => void; notice: string | null }) {
  const [query, setQuery] = useState('');
  const [group, setGroup] = useState('computers');
  const categories = GROUPS.find((g) => g.id === group)?.categories ?? [];
  const needle = query.trim().toLowerCase();
  const packs = [...props.own, ...Object.values(props.catalog.devices)]
    .filter((pack) => (needle ? [pack.label, ...(pack.aka ?? [])].some((text) => text.toLowerCase().includes(needle)) : categories.includes(pack.category)))
    .sort((a, b) => a.label.localeCompare(b.label));

  return (
    <aside className="palette" aria-label="Parts">
      <h2>Parts</h2>
      <input className="input" type="search" placeholder="Search parts" aria-label="Search parts" value={query} onChange={(event) => setQuery(event.target.value)} />
      <div className="palette__groups" role="tablist" aria-label="Kinds of part">
        {GROUPS.map((g) => (
          <button key={g.id} type="button" role="tab" aria-selected={g.id === group && !needle} className="pill" onClick={() => { setGroup(g.id); setQuery(''); }}>
            {g.label}
          </button>
        ))}
      </div>
      {props.notice && (
        <p className="palette__notice" role="status">
          {props.notice}
        </p>
      )}
      <ul className="palette__list">
        {packs.map((pack) => (
          <li key={pack.id}>
            <button type="button" className="part" data-testid={`part-${pack.id}`} onClick={() => props.onAdd(pack)}>
              <span className="part__label">{pack.label}</span>
              <span className="part__facts">{facts(pack) || 'No details known'}</span>
              <span className="part__kind">{KIND_WORD[pack.kind]}</span>
            </button>
          </li>
        ))}
        {packs.length === 0 && <li className="palette__empty">Nothing matches.</li>}
      </ul>
      <CustomDevice taken={[...props.own.map((p) => p.id), ...Object.keys(props.catalog.devices)]} onCreate={props.onCustom} />
    </aside>
  );
}

type YesNo = 'yes' | 'no' | '';

function CustomDevice({ taken, onCreate }: { taken: string[]; onCreate: (pack: DevicePack) => void }) {
  const [label, setLabel] = useState('');
  const [arch, setArch] = useState('');
  const [ram, setRam] = useState<number | undefined>();
  const [wired, setWired] = useState<YesNo>('');
  const [wifi, setWifi] = useState<YesNo>('');
  const [battery, setBattery] = useState(false);
  const [wh, setWh] = useState<number | undefined>();
  const [idle, setIdle] = useState<number | undefined>();

  const create = () => {
    let id = `my-${label.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'device'}`;
    for (let n = 2; taken.includes(id); n += 1) id = `${id.replace(/-\d+$/, '')}-${n}`;
    // Whatever is left blank stays unknown. The core treats it as unknown, never as a pass.
    const pack: DevicePack = {
      id,
      kind: 'learner',
      category: 'computer',
      label: label.trim() || 'My device',
      compute: { ...(arch ? { arch: arch as Arch } : {}), ...(ram !== undefined ? { ram_gb: ram } : {}) },
      network: {
        ...(wired ? { nic: wired === 'yes' ? [{ speed_mbps: 'unknown' as const }] : [] } : {}),
        ...(wifi ? { wifi: (wifi === 'yes' ? 'wifi5' : 'none') as WifiGen } : {}),
      },
      ...(battery ? { battery: wh !== undefined ? { wh } : {} } : {}),
      ...(idle !== undefined ? { power: { idle_w: idle } } : {}),
    };
    onCreate(pack);
    setLabel('');
  };
  const tri = [
    { value: 'yes', label: 'Yes' },
    { value: 'no', label: 'No' },
  ];
  return (
    <details className="custom">
      <summary data-testid="custom-open">My device isn't listed</summary>
      <p>Tell us what you know. Anything you leave blank stays unknown, and the report will say so.</p>
      <Text label="What do you call it?" value={label} onChange={setLabel} testid="custom-label" />
      <Select label="Processor" value={arch} none="Don't know" options={[{ value: 'x86_64', label: 'Intel or AMD (x86)' }, { value: 'aarch64', label: 'ARM, 64-bit' }]} onChange={setArch} testid="custom-arch" />
      <Num label="Memory, in GB" value={ram} onChange={setRam} testid="custom-ram" />
      <Select label="Wired Ethernet port" value={wired} none="Don't know" options={tri} onChange={(v) => setWired(v as YesNo)} testid="custom-wired" />
      <Select label="Wi-Fi" value={wifi} none="Don't know" options={tri} onChange={(v) => setWifi(v as YesNo)} />
      <Check label="It has a battery" checked={battery} onChange={setBattery} />
      {battery && <Num label="Battery size, in Wh, if you know it" value={wh} onChange={setWh} />}
      <Num label="Idle power, in watts, if you know it" value={idle} onChange={setIdle} />
      <button type="button" className="btn" data-testid="custom-add" onClick={create}>
        Add my device
      </button>
    </details>
  );
}
