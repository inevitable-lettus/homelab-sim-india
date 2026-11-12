// What the learner wants from the build. Each goal becomes an intent the
// core checks; this panel only collects the parameters.
import { useState } from 'react';
import { TrashIcon } from '../components/icons';
import { goalText, isHost, uniqueId } from '../lib/build';
import type { Catalog } from '../lib/packs';
import type { HomePlan, Id, Intent, World } from '../lib/types';
import { Check, Num, Section, Select, type Option } from './fields';

type Kind = Intent['kind'];

const KINDS: { value: Kind; label: string }[] = [
  { value: 'run', label: 'Run a service on a machine' },
  { value: 'stream', label: 'Watch video at home' },
  { value: 'reach', label: 'Open a service from my phone on mobile data' },
  { value: 'home_dns', label: 'Ad-blocking for the whole house' },
  { value: 'store', label: 'Keep files' },
  { value: 'survive_outage', label: 'Survive a power cut' },
  { value: 'yearly_cost', label: 'Keep yearly electricity under a limit' },
];

export function Goals({ world, catalog, edit }: { world: World; catalog: Catalog; edit: (change: (world: World) => World) => void }) {
  const [kind, setKind] = useState<Kind>('run');
  const [target, setTarget] = useState('');
  const [client, setClient] = useState('');
  const [format, setFormat] = useState('');
  const [router, setRouter] = useState('');
  const [outage, setOutage] = useState('');
  const [gb, setGb] = useState<number | undefined>(100);
  const [copies, setCopies] = useState<number | undefined>(1);
  const [limit, setLimit] = useState<number | undefined>(1500);

  const intents = world.intents ?? [];
  const servers = world.nodes.filter(isHost).filter((n) => n.role !== 'client');
  const services: Option[] = servers.flatMap((host) => (host.services ?? []).map((s) => ({ value: `${host.id}/${s.id}`, label: `${s.label} on ${host.label}` })));
  const machines: Option[] = servers.filter((h) => !h.runs_on).map((h) => ({ value: h.id, label: h.label }));
  const clients: Option[] = world.nodes.filter(isHost).filter((n) => n.role === 'client' && n.id !== 'away-phone').map((n) => ({ value: n.id, label: n.label }));
  const routers: Option[] = world.nodes.filter((n) => n.kind === 'router').map((n) => ({ value: n.id, label: n.label }));
  const formats: Option[] = Object.entries(catalog.media ?? {}).map(([value, f]) => ({ value, label: f.label }));
  const outages: Option[] = Object.entries(catalog.outages ?? {}).map(([value, o]) => ({ value, label: o.label }));
  const keepable = intents.filter((i) => i.kind !== 'survive_outage' && i.kind !== 'yearly_cost' && i.kind !== 'store');

  // Sensible defaults from the board: the only service, the only TV, the only router.
  const chosen = target || services[0]?.value || '';
  const [host = '', service = ''] = chosen.split('/');
  const theClient = client || clients[0]?.value || '';
  const theFormat = format || formats[0]?.value || '';
  const theRouter = router || routers[0]?.value || '';
  const theOutage = outage || world.home?.outage || outages[0]?.value || '';
  const theMachine = machines.some((m) => m.value === target) ? target : (machines[0]?.value ?? '');

  const needsService = kind === 'run' || kind === 'stream' || kind === 'reach' || kind === 'home_dns';
  const missing =
    needsService && !chosen
      ? 'Add a machine with a service first.'
      : kind === 'stream' && !theClient
        ? 'Add a TV or another client first.'
        : (kind === 'reach' || kind === 'home_dns') && !theRouter
          ? 'Add a router first.'
          : kind === 'store' && !theMachine
            ? 'Add a machine first.'
            : kind === 'survive_outage' && keepable.length === 0
              ? 'Add a goal to keep going through the cut first.'
              : kind === 'survive_outage' && !theOutage
                ? 'No power-cut scenarios are loaded.'
                : null;

  const add = () => {
    const id = uniqueId(kind.replace(/_/g, '-'), intents.map((i) => i.id));
    let intent: Intent;
    switch (kind) {
      case 'run':
        intent = { kind, id, host, service };
        break;
      case 'stream':
        intent = { kind, id, client: theClient, host, service, format: theFormat };
        break;
      case 'reach':
        intent = { kind, id, from: 'away-phone', host, service, via: { kind: 'public_of', node: theRouter } };
        break;
      case 'home_dns':
        intent = { kind, id, host, service, router: theRouter };
        break;
      case 'store':
        intent = { kind, id, host: theMachine, gb: gb ?? 1, copies: Math.max(1, copies ?? 1) };
        break;
      case 'survive_outage':
        intent = { kind, id, outage: theOutage, keep: keepable.map((i) => i.id) };
        break;
      case 'yearly_cost':
        intent = { kind, id, max_paise: (limit ?? 0) * 100 };
        break;
    }
    edit((w) => ({ ...w, intents: [...(w.intents ?? []), intent] }));
  };
  const remove = (id: Id) =>
    edit((w) => ({
      ...w,
      intents: (w.intents ?? []).filter((i) => i.id !== id).map((i) => (i.kind === 'survive_outage' ? { ...i, keep: i.keep.filter((k) => k !== id) } : i)),
    }));
  const home = world.home ?? {};
  const setHome = (change: Partial<HomePlan>) => edit((w) => ({ ...w, home: { ...w.home, ...change } }));
  const tariff = home.tariff ? catalog.tariffs?.[home.tariff] : undefined;

  return (
    <div className="bgoals" data-testid="goals">
      <Section title="Your goals">
        <ul className="rows">
          {intents.map((intent) => (
            <li key={intent.id} data-testid={`goal-${intent.id}`}>
              <span>{goalText(intent, world, catalog)}</span>
              <button type="button" className="iconbtn" aria-label={`Remove the goal: ${goalText(intent, world, catalog)}`} onClick={() => remove(intent.id)}>
                <TrashIcon size={16} />
              </button>
            </li>
          ))}
          {intents.length === 0 && <li className="rows__empty">No goals yet. Add one and the report will say whether the build meets it.</li>}
        </ul>
      </Section>
      <Section title="Add a goal">
        <Select label="I want to" value={kind} options={KINDS} onChange={(value) => { setKind(value as Kind); setTarget(''); }} testid="goal-kind" />
        {needsService && services.length > 0 && <Select label="Which service" value={chosen} options={services} onChange={setTarget} testid="goal-service" />}
        {kind === 'stream' && clients.length > 0 && <Select label="On which screen" value={theClient} options={clients} onChange={setClient} />}
        {kind === 'stream' && <Select label="What kind of video" value={theFormat} options={formats} onChange={setFormat} />}
        {(kind === 'home_dns' || kind === 'reach') && routers.length > 1 && <Select label="Through which router" value={theRouter} options={routers} onChange={setRouter} />}
        {kind === 'store' && machines.length > 0 && <Select label="On which machine" value={theMachine} options={machines} onChange={setTarget} />}
        {kind === 'store' && <Num label="How much, in GB" value={gb} min={1} onChange={setGb} testid="goal-gb" />}
        {kind === 'store' && <Num label="How many copies" value={copies} min={1} max={4} onChange={setCopies} />}
        {kind === 'survive_outage' && <Select label="Which power cut" value={theOutage} options={outages} onChange={setOutage} testid="goal-outage" />}
        {kind === 'yearly_cost' && <Num label="Limit, in rupees a year" value={limit} onChange={setLimit} testid="goal-limit" />}
        {missing ? (
          <p className="bsec__note" role="status">
            {missing}
          </p>
        ) : (
          <button type="button" className="btn btn--primary" data-testid="goal-add" onClick={add}>
            Add this goal
          </button>
        )}
      </Section>
      <Section title="Your electricity">
        <Select label="Tariff" value={home.tariff ?? ''} none="Not chosen" options={Object.entries(catalog.tariffs ?? {}).map(([value, t]) => ({ value, label: `${t.discom}${t.teaching ? ' (teaching)' : ''}` }))} onChange={(value) => setHome({ tariff: value || undefined })} testid="tariff" />
        <Num label="Units your home already uses each month (kWh)" value={home.base_kwh_month} hint="Left blank, the report assumes 150 and says so." onChange={(base_kwh_month) => setHome({ base_kwh_month })} />
        {tariff?.free_units && <Check label={`This applies to my home: ${tariff.free_units.condition}`} checked={!!home.free_units_apply} onChange={(free_units_apply) => setHome({ free_units_apply })} />}
        <Num label="Or your own rate, in paise per kWh" value={home.rate_paise_per_kwh} hint="When set, this replaces the tariff's slabs." onChange={(rate_paise_per_kwh) => setHome({ rate_paise_per_kwh })} />
      </Section>
    </div>
  );
}
