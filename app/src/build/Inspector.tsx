// Edit one part of the build. Every control changes the world; the core
// then says what that did.
import { useState } from 'react';
import { TrashIcon } from '../components/icons';
import { devicesOf, isHost, rupees, SCAFFOLD, uniqueId } from '../lib/build';
import type { Catalog, Category, DevicePack } from '../lib/packs';
import type { HostNode, Id, LineMode, Medium, Mount, PortForward, Price, Proto, Report, RouterNode, World, WorldNode } from '../lib/types';
import { Check, Num, Section, Select, Text, type Option } from './fields';

type Edit = (change: (world: World) => World) => void;

const mapNode = (world: World, id: Id, change: (node: WorldNode) => WorldNode): World => ({
  ...world,
  nodes: world.nodes.map((node) => (node.id === id ? change(node) : node)),
});

const options = (packs: Record<Id, { label: string }> | undefined): Option[] =>
  Object.entries(packs ?? {})
    .map(([value, pack]) => ({ value, label: pack.label }))
    .sort((a, b) => a.label.localeCompare(b.label));

const deviceOptions = (devices: Record<Id, DevicePack>, categories: Category[]): Option[] =>
  Object.values(devices)
    .filter((pack) => categories.includes(pack.category))
    .map((pack) => ({ value: pack.id, label: pack.kind === 'learner' ? `${pack.label} (yours)` : pack.label }))
    .sort((a, b) => a.label.localeCompare(b.label));

function PriceField({ price, onChange }: { price: Price | undefined; onChange: (price: Price | undefined) => void }) {
  return (
    <Section title="Price">
      <p className="bsec__note">Your own figure. We don't ship prices, and yours stay in this browser.</p>
      <Num label="What it costs, in rupees" value={price ? Math.round(price.paise / 100) : undefined} onChange={(value) => onChange(value === undefined ? undefined : { paise: value * 100, owned: price?.owned })} testid="price" />
      {price && <Check label="I already own this" checked={!!price.owned} onChange={(owned) => onChange({ ...price, owned })} />}
      {price && <p className="bsec__note">{rupees(price.paise)}</p>}
    </Section>
  );
}

function Cables({ world, node, edit, onConnect }: { world: World; node: WorldNode; edit: Edit; onConnect: (id: Id) => void }) {
  const label = (id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id;
  const links = world.links.filter((link) => link.a === node.id || link.b === node.id);
  return (
    <Section title="Connections">
      <ul className="rows">
        {links.map((link) => (
          <li key={link.id}>
            <span>
              {link.medium === 'wifi' ? 'Wi-Fi' : link.medium} to {label(link.a === node.id ? link.b : link.a)}
            </span>
            <button type="button" className="iconbtn" aria-label={`Remove the connection to ${label(link.a === node.id ? link.b : link.a)}`} onClick={() => edit((w) => ({ ...w, links: w.links.filter((l) => l.id !== link.id) }))}>
              <TrashIcon size={16} />
            </button>
          </li>
        ))}
        {links.length === 0 && <li className="rows__empty">Not connected to anything.</li>}
      </ul>
      <button type="button" className="btn" data-testid="connect" onClick={() => onConnect(node.id)}>
        Connect to…
      </button>
    </Section>
  );
}

function Machine({ world, catalog, node, edit, report }: { world: World; catalog: Catalog; node: HostNode; edit: Edit; report: Report | null }) {
  const devices = devicesOf(world, catalog);
  const set = (change: (host: HostNode) => HostNode) => edit((w) => mapNode(w, node.id, (n) => (isHost(n) ? change(n) : n)));
  const guest = !!node.runs_on;
  const [service, setService] = useState('');
  const [part, setPart] = useState('');
  const parts = node.parts ?? [];
  const holders = parts.filter((p) => ['enclosure', 'usb_hub'].includes(devices[p.pack]?.category ?? ''));
  const settings = node.settings ?? {};

  const addService = () => {
    const pack = catalog.services?.[service];
    if (!pack) return;
    const port = pack.ports[0];
    const method = guest && node.guest !== 'vm' ? 'native' : (pack.install[0]?.method ?? 'native');
    set((host) => ({
      ...host,
      services: [...(host.services ?? []), { id: uniqueId(pack.id, (host.services ?? []).map((s) => s.id)), label: pack.label, proto: port?.proto ?? 'tcp', port: port?.port ?? 80, app: pack.id, install: method }],
    }));
    setService('');
  };
  const addGuest = (kind: 'vm' | 'lxc' | 'container') =>
    edit((w) => {
      const id = uniqueId(kind, w.nodes.map((n) => n.id));
      const label = kind === 'vm' ? 'Virtual machine' : kind === 'lxc' ? 'LXC container' : 'Container';
      const added: HostNode = { kind: 'host', id, label, runs_on: node.id, guest: kind, alloc: { ram_mb: 1024 } };
      return { ...w, nodes: [...w.nodes, added] };
    });

  return (
    <>
      {!guest && (
        <Section title="Device">
          <Select label="What it is" value={node.device ?? ''} none="Not chosen" options={deviceOptions(devices, [node.role === 'client' ? 'client' : 'computer'])} onChange={(device) => set((h) => ({ ...h, device: device || undefined }))} testid="device" />
        </Section>
      )}
      {node.role !== 'client' && (
        <>
          <Section title="System">
            {guest && <Num label="Memory given to it, in MB" value={node.alloc?.ram_mb} onChange={(ram_mb) => set((h) => ({ ...h, alloc: { ...h.alloc, ram_mb } }))} testid="alloc-ram" />}
            {(!guest || node.guest === 'vm') && <Select label="Operating system" value={node.os ?? ''} none="Not chosen" options={options(catalog.os)} onChange={(os) => set((h) => ({ ...h, os: os || undefined }))} testid="os" />}
            {(!guest || node.guest === 'vm') && <Select label="Container runtime" value={node.runtime ?? ''} none="None" options={options(catalog.runtimes)} onChange={(runtime) => set((h) => ({ ...h, runtime: runtime || undefined }))} testid="runtime" />}
          </Section>
          <Section title="Services">
            <ul className="rows">
              {(node.services ?? []).map((s) => (
                <li key={s.id}>
                  <span>
                    {s.label} <small>{s.proto.toUpperCase()} {s.port}{s.install ? `, ${s.install}` : ''}</small>
                  </span>
                  {s.app && catalog.services?.[s.app] && (
                    <select className="input input--small" aria-label={`How ${s.label} is installed`} value={s.install ?? 'native'} onChange={(event) => set((h) => ({ ...h, services: h.services?.map((x) => (x.id === s.id ? { ...x, install: event.target.value as typeof x.install } : x)) }))}>
                      {[...new Set((catalog.services[s.app]?.install ?? []).map((i) => i.method))].map((method) => (
                        <option key={method}>{method}</option>
                      ))}
                    </select>
                  )}
                  <button type="button" className="iconbtn" aria-label={`Remove ${s.label}`} onClick={() => edit((w) => ({ ...mapNode(w, node.id, (n) => (isHost(n) ? { ...n, services: n.services?.filter((x) => x.id !== s.id) } : n)), intents: w.intents?.filter((i) => !('service' in i && i.host === node.id && i.service === s.id)) }))}>
                    <TrashIcon size={16} />
                  </button>
                </li>
              ))}
              {(node.services ?? []).length === 0 && <li className="rows__empty">Nothing installed.</li>}
            </ul>
            <div className="addrow">
              <Select label="Add a service" value={service} none="Choose…" options={options(catalog.services)} onChange={setService} testid="service" />
              <button type="button" className="btn" disabled={!service} data-testid="service-add" onClick={addService}>
                Add
              </button>
            </div>
          </Section>
        </>
      )}
      {!guest && node.role !== 'client' && (
        <>
          <Section title="Parts">
            <ul className="rows">
              {parts.map((p) => (
                <li key={p.id}>
                  <span>{devices[p.pack]?.label ?? p.pack}</span>
                  {holders.some((h) => h.id !== p.id) && (
                    <select className="input input--small" aria-label={`Where ${devices[p.pack]?.label ?? p.pack} is fitted`} value={p.into ?? ''} onChange={(event) => set((h) => ({ ...h, parts: h.parts?.map((x) => (x.id === p.id ? { ...x, into: event.target.value || undefined } : x)) }))}>
                      <option value="">In the machine</option>
                      {holders.filter((h) => h.id !== p.id).map((h) => (
                        <option key={h.id} value={h.id}>
                          In: {devices[h.pack]?.label ?? h.pack}
                        </option>
                      ))}
                    </select>
                  )}
                  <button type="button" className="iconbtn" aria-label={`Remove ${devices[p.pack]?.label ?? p.pack}`} onClick={() => set((h) => ({ ...h, parts: h.parts?.filter((x) => x.id !== p.id).map((x) => (x.into === p.id ? { ...x, into: undefined } : x)) }))}>
                    <TrashIcon size={16} />
                  </button>
                </li>
              ))}
              {parts.length === 0 && <li className="rows__empty">No drives or adapters fitted.</li>}
            </ul>
            <div className="addrow">
              <Select label="Fit a part" value={part} none="Choose…" options={deviceOptions(devices, ['drive', 'enclosure', 'usb_nic', 'usb_hub'])} onChange={setPart} testid="part" />
              <button type="button" className="btn" disabled={!part} data-testid="part-add" onClick={() => { set((h) => ({ ...h, parts: [...(h.parts ?? []), { id: uniqueId(part, (h.parts ?? []).map((x) => x.id)), pack: part } satisfies Mount] })); setPart(''); }}>
                Add
              </button>
            </div>
          </Section>
          <Section title="Guests">
            <div className="btnrow">
              <button type="button" className="btn" data-testid="add-vm" onClick={() => addGuest('vm')}>Add a VM</button>
              <button type="button" className="btn" onClick={() => addGuest('lxc')}>Add an LXC</button>
              <button type="button" className="btn" onClick={() => addGuest('container')}>Add a container</button>
            </div>
          </Section>
          <Section title="Settings">
            <Check label="Runs with the lid closed" checked={!!settings.lid_closed} onChange={(lid_closed) => set((h) => ({ ...h, settings: { ...h.settings, lid_closed } }))} testid="lid" />
            <Check label="Battery charge is capped" checked={!!settings.charge_limit} onChange={(charge_limit) => set((h) => ({ ...h, settings: { ...h.settings, charge_limit } }))} />
            <Check label="Starts again by itself when power returns" checked={!!settings.ac_power_recovery} onChange={(ac_power_recovery) => set((h) => ({ ...h, settings: { ...h.settings, ac_power_recovery } }))} testid="recovery" />
            <Check label="Left on all day" checked={settings.always_on !== false} onChange={(always_on) => set((h) => ({ ...h, settings: { ...h.settings, always_on } }))} />
            <Num label="Share of the time it is busy, in percent" value={node.busy_permille === undefined ? undefined : Math.round(node.busy_permille / 10)} max={100} hint="Left blank, the report assumes 10% and says so." onChange={(value) => set((h) => ({ ...h, busy_permille: value === undefined ? undefined : Math.min(1000, value * 10) }))} />
          </Section>
          <PriceField price={node.price} onChange={(price) => set((h) => ({ ...h, price }))} />
        </>
      )}
      {report?.facts.nodes.find((n) => n.id === node.id)?.ipv4 && <p className="bsec__note">Address in this model: {report.facts.nodes.find((n) => n.id === node.id)?.ipv4}</p>}
    </>
  );
}

function Router({ world, catalog, node, edit, report }: { world: World; catalog: Catalog; node: RouterNode; edit: Edit; report: Report | null }) {
  const devices = devicesOf(world, catalog);
  const set = (change: (router: RouterNode) => RouterNode) => edit((w) => mapNode(w, node.id, (n) => (n.kind === 'router' ? change(n) : n)));
  const targets = world.nodes.flatMap((n) => (isHost(n) && n.role !== 'client' ? (n.services ?? []).map((s) => ({ host: n, service: s })) : []));
  const [pick, setPick] = useState('');
  const forwards = node.port_forwards ?? [];
  const add = () => {
    const target = targets.find((t) => `${t.host.id}/${t.service.id}` === pick);
    const ip = report?.facts.nodes.find((n) => n.id === target?.host.id)?.ipv4;
    if (!target || !ip) return;
    const forward: PortForward = { id: uniqueId(target.service.id, forwards.map((f) => f.id)), enabled: true, proto: target.service.proto as Proto, external_port: target.service.port, to_ip: ip, internal_port: target.service.port };
    set((r) => ({ ...r, port_forwards: [...forwards, forward] }));
    setPick('');
  };
  return (
    <>
      <Section title="Device">
        <Select label="Which router" value={node.device ?? ''} none="Not chosen" options={deviceOptions(devices, ['router'])} onChange={(device) => set((r) => ({ ...r, device: device || undefined }))} testid="device" />
      </Section>
      <Section title="Port forwards">
        <ul className="rows">
          {forwards.map((f) => (
            <li key={f.id}>
              <span>
                {f.proto.toUpperCase()} {f.external_port} to {f.to_ip}:{f.internal_port}
              </span>
              <button type="button" className="iconbtn" aria-label={`Remove the forward for port ${f.external_port}`} onClick={() => set((r) => ({ ...r, port_forwards: forwards.filter((x) => x.id !== f.id) }))}>
                <TrashIcon size={16} />
              </button>
            </li>
          ))}
          {forwards.length === 0 && <li className="rows__empty">No ports are forwarded.</li>}
        </ul>
        <div className="addrow">
          <Select label="Forward a port to" value={pick} none="Choose a service…" options={targets.map((t) => ({ value: `${t.host.id}/${t.service.id}`, label: `${t.service.label} on ${t.host.label}` }))} onChange={setPick} testid="forward" />
          <button type="button" className="btn" disabled={!pick} data-testid="forward-add" onClick={add}>
            Add
          </button>
        </div>
      </Section>
      <PriceField price={node.price} onChange={(price) => set((r) => ({ ...r, price }))} />
    </>
  );
}

export function Inspector(props: { world: World; catalog: Catalog; report: Report | null; id: Id; edit: Edit; onConnect: (id: Id) => void; onDelete: (id: Id) => void }) {
  const { world, catalog, edit } = props;
  const node = world.nodes.find((n) => n.id === props.id);
  if (!node) return null;
  const devices = devicesOf(world, catalog);
  const locked = SCAFFOLD.includes(node.id);
  const cabled = node.kind !== 'power' && !(isHost(node) && node.runs_on);
  return (
    <div className="binspector" data-testid="inspector">
      <Text label="Name" value={node.label} onChange={(label) => edit((w) => mapNode(w, node.id, (n) => ({ ...n, label })))} testid="name" />
      {node.kind === 'host' && <Machine world={world} catalog={catalog} node={node} edit={edit} report={props.report} />}
      {node.kind === 'router' && <Router world={world} catalog={catalog} node={node} edit={edit} report={props.report} />}
      {node.kind === 'isp' && (
        <Section title="Line">
          <Select
            label="What kind of address the line gets"
            value={node.ipv4}
            options={[
              { value: 'unknown', label: "I don't know" },
              { value: 'public', label: 'A public address' },
              { value: 'cgnat', label: 'A shared address (carrier NAT)' },
            ]}
            onChange={(ipv4) => edit((w) => mapNode(w, node.id, (n) => (n.kind === 'isp' ? { ...n, ipv4: ipv4 as LineMode } : n)))}
            testid="line"
          />
        </Section>
      )}
      {(node.kind === 'switch' || node.kind === 'power') && (
        <>
          <Section title="Device">
            <Select label="Which one" value={node.device ?? ''} none="Not chosen" options={deviceOptions(devices, node.kind === 'switch' ? ['switch'] : ['ups', 'inverter', 'dc_ups'])} onChange={(device) => edit((w) => mapNode(w, node.id, (n) => (n.kind === node.kind ? { ...n, device: device || undefined } : n)))} testid="device" />
          </Section>
          {node.kind === 'power' && (
            <Section title="What it feeds">
              {world.nodes
                .filter((n) => n.kind === 'router' || n.kind === 'switch' || (isHost(n) && !n.runs_on && !SCAFFOLD.includes(n.id)))
                .map((n) => {
                  const fed = (world.power_feeds ?? []).some((f) => f.from === node.id && f.to === n.id);
                  return (
                    <Check key={n.id} label={n.label} checked={fed} testid={`feed-${n.id}`} onChange={(on) => edit((w) => ({ ...w, power_feeds: on ? [...(w.power_feeds ?? []), { from: node.id, to: n.id }] : (w.power_feeds ?? []).filter((f) => !(f.from === node.id && f.to === n.id)) }))} />
                  );
                })}
            </Section>
          )}
          <PriceField price={node.price} onChange={(price) => edit((w) => mapNode(w, node.id, (n) => (n.kind === node.kind ? { ...n, price } : n)))} />
        </>
      )}
      {cabled && <Cables world={world} node={node} edit={edit} onConnect={props.onConnect} />}
      {locked ? (
        <p className="bsec__note">This is part of the outside world. It can be moved, not removed.</p>
      ) : (
        <button type="button" className="btn btn--danger" data-testid="delete" onClick={() => props.onDelete(node.id)}>
          Remove from the build
        </button>
      )}
    </div>
  );
}

/** The cable to use between two parts, when there is only one sensible answer. */
export function cableChoices(world: World, a: Id, b: Id): Medium[] {
  const kind = (id: Id) => world.nodes.find((n) => n.id === id)?.kind;
  const kinds = [kind(a), kind(b)];
  if (kinds.includes('isp')) return kinds.includes('router') ? ['fibre'] : [];
  if (kinds.includes('internet') || kinds.includes('power')) return [];
  if (!kinds.includes('router') && !kinds.includes('switch')) return [];
  return kinds.includes('switch') ? ['ethernet'] : ['ethernet', 'wifi'];
}
