// The sandbox: "build your own". The list of builds, the import route and
// the editor. The core decides everything shown as a result.
import { useEffect, useMemo, useRef, useState } from 'react';
import { Board, placed } from '../build/Board';
import { Goals } from '../build/Goals';
import { cableChoices, Inspector } from '../build/Inspector';
import { Palette } from '../build/Palette';
import { ReportPanel } from '../build/Report';
import { ArrowLeftIcon, TrashIcon } from '../components/icons';
import {
  deleteBuild,
  emptyBuild,
  freeSlot,
  fromEnvelope,
  fromSharePayload,
  fromTemplate,
  getBuild,
  isHost,
  linkBetween,
  listBuilds,
  problemText,
  removeNode,
  saveBuild,
  SHARE_LIMIT,
  sharePayload,
  storageWorks,
  toEnvelope,
  uniqueId,
  type SavedBuild,
} from '../lib/build';
import type { Catalog, DevicePack } from '../lib/packs';
import { href } from '../lib/route';
import { evaluate, loadCatalog, type WorldProblem } from '../lib/sim';
import type { Bundle, Id, Medium, Position, Report, World, WorldNode } from '../lib/types';
import '../styles/build.css';

function useCatalog(): { catalog: Catalog | null; error: string | null } {
  const [state, setState] = useState<{ catalog: Catalog | null; error: string | null }>({ catalog: null, error: null });
  useEffect(() => {
    let active = true;
    loadCatalog().then(
      (catalog) => active && setState({ catalog, error: null }),
      (error: unknown) => active && setState({ catalog: null, error: error instanceof Error ? error.message : String(error) }),
    );
    return () => {
      active = false;
    };
  }, []);
  return state;
}

function Shell({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="page page--wide">
      <a className="page__back" href={href.home}>
        <ArrowLeftIcon size={16} /> Home
      </a>
      <main>
        <h1>{title}</h1>
        {children}
      </main>
    </div>
  );
}

function Waiting({ error }: { error: string | null }) {
  return error ? (
    <p role="alert">The parts catalog did not load ({error}). Reloading the page often helps.</p>
  ) : (
    <p role="status">Loading the parts catalog…</p>
  );
}

const StorageNotice = () =>
  storageWorks ? null : (
    <p className="notice" role="status" data-testid="storage-notice">
      This browser is not letting the site save anything, so your build lasts only until you close the tab. Export it to a file to keep it.
    </p>
  );

function Problems({ problems }: { problems: WorldProblem[] }) {
  return (
    <div role="alert" data-testid="import-problems">
      <p>That build could not be loaded:</p>
      <ul>
        {problems.slice(0, 8).map((problem, index) => (
          <li key={index}>{problemText(problem)}</li>
        ))}
      </ul>
    </div>
  );
}

const open = (build: SavedBuild) => {
  saveBuild(build);
  location.hash = href.build(build.id);
};

export function Builds() {
  const { catalog, error } = useCatalog();
  const [builds, setBuilds] = useState(listBuilds);
  const [problems, setProblems] = useState<WorldProblem[] | null>(null);
  if (!catalog) {
    return (
      <Shell title="Build your own">
        <Waiting error={error} />
      </Shell>
    );
  }
  const templates = Object.values(catalog.templates ?? {});
  const importFile = async (file: File | undefined) => {
    if (!file) return;
    const result = fromEnvelope(await file.text());
    if (result.ok) open(result.build);
    else setProblems(result.problems);
  };
  return (
    <Shell title="Build your own">
      <p className="page__lede">Put together the machines and network you have, or are thinking of buying, and see what works, what fits and what it costs to run. Everything stays in this browser.</p>
      <StorageNotice />
      <div className="btnrow">
        <button type="button" className="btn btn--primary" data-testid="start-empty" onClick={() => open(emptyBuild())}>
          Start with an empty board
        </button>
        <label className="btn">
          Open a build file
          <input type="file" accept="application/json,.json" className="sr-only" data-testid="import-file" onChange={(event) => void importFile(event.target.files?.[0])} />
        </label>
      </div>
      {problems && <Problems problems={problems} />}
      {builds.length > 0 && (
        <>
          <h2>Your builds</h2>
          <ul className="buildlist">
            {builds.map((build) => (
              <li key={build.id}>
                <a href={href.build(build.id)} data-testid={`build-${build.id}`}>
                  <strong>{build.name}</strong>
                  <span>Changed {new Date(build.updated).toLocaleDateString()}</span>
                </a>
                <button type="button" className="iconbtn" aria-label={`Delete ${build.name}`} onClick={() => { deleteBuild(build.id); setBuilds(listBuilds()); }}>
                  <TrashIcon size={16} />
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
      <h2>Start from an example</h2>
      <ul className="buildlist">
        {templates.map((template) => (
          <li key={template.id}>
            <button type="button" data-testid={`template-${template.id}`} onClick={() => open(fromTemplate(template))}>
              <strong>{template.title}</strong>
              <span>{template.summary}</span>
            </button>
          </li>
        ))}
      </ul>
    </Shell>
  );
}

export function BuildImport({ payload }: { payload: string }) {
  const { catalog, error } = useCatalog();
  const [problems, setProblems] = useState<WorldProblem[] | null>(null);
  useEffect(() => {
    if (!catalog) return;
    let active = true;
    void fromSharePayload(payload).then((result) => {
      if (!active) return;
      if (result.ok) {
        saveBuild(result.build);
        location.replace(href.build(result.build.id));
      } else setProblems(result.problems);
    });
    return () => {
      active = false;
    };
  }, [catalog, payload]);
  return (
    <Shell title="Opening a shared build">
      {!catalog && <Waiting error={error} />}
      {problems && <Problems problems={problems} />}
      {problems && <a href={href.builds}>Go to your builds</a>}
    </Shell>
  );
}

type Tab = 'goals' | 'report' | 'inspect';

export function BuildEditor({ bundle, id }: { bundle: Bundle; id: Id }) {
  const { catalog, error } = useCatalog();
  const [build, setBuild] = useState(() => getBuild(id));
  const [selected, setSelected] = useState<Id | null>(null);
  const [tab, setTab] = useState<Tab>('report');
  const [connecting, setConnecting] = useState<Id | null>(null);
  const [cable, setCable] = useState<{ a: Id; b: Id; choices: Medium[] } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [showPower, setShowPower] = useState(true);
  const [prices, setPrices] = useState(false);
  const [shared, setShared] = useState<string | null>(null);
  const first = useRef(true);

  useEffect(() => {
    if (first.current) first.current = false;
    else if (build) saveBuild(build);
  }, [build]);

  const world = build?.world;
  const result = useMemo((): { report: Report | null; error: string | null } => {
    if (!world || !catalog) return { report: null, error: null };
    try {
      return { report: evaluate(world), error: null };
    } catch (failure) {
      return { report: null, error: failure instanceof Error ? failure.message : String(failure) };
    }
  }, [world, catalog]);

  if (!build || !world) {
    return (
      <Shell title="That build is not in this browser">
        <p>Builds are kept in the browser that made them. To move one, export it to a file or copy its link.</p>
        <a href={href.builds}>Go to your builds</a>
      </Shell>
    );
  }
  if (!catalog) {
    return (
      <Shell title={build.name}>
        <Waiting error={error} />
      </Shell>
    );
  }

  const edit = (change: (world: World) => World) => setBuild((b) => (b ? { ...b, world: change(b.world) } : b));
  const layout = placed(world, build.layout.wide);
  const move = (node: Id, at: Position) => setBuild((b) => (b ? { ...b, layout: { ...b.layout, wide: { ...placed(b.world, b.layout.wide), [node]: at } } } : b));
  const place = (node: WorldNode, extra?: (world: World) => World) => {
    setBuild((b) => {
      if (!b) return b;
      const wide = placed(b.world, b.layout.wide);
      const next = { ...b.world, nodes: [...b.world.nodes, node] };
      return { ...b, world: extra ? extra(next) : next, layout: { ...b.layout, wide: { ...wide, [node.id]: freeSlot(wide) } } };
    });
    setSelected(node.id);
    setTab('inspect');
    setNotice(null);
  };

  const add = (pack: DevicePack) => {
    const ids = world.nodes.map((n) => n.id);
    const id = uniqueId(pack.label, ids);
    const base = { id, label: pack.label, device: pack.id };
    switch (pack.category) {
      case 'computer':
        return place({ kind: 'host', ...base });
      case 'client':
        return place({ kind: 'host', role: 'client', ...base });
      case 'router': {
        const used = world.nodes.flatMap((n) => (n.kind === 'router' ? [n.lan] : []));
        const lan = [1, 2, 3, 4, 5, 6, 7, 8].map((n) => `192.168.${n}.0/24`).find((net) => !used.includes(net)) ?? '192.168.9.0/24';
        const uplinked = world.links.some((l) => l.a === 'home-isp' || l.b === 'home-isp');
        return place({ kind: 'router', lan, udp_timeout_s: 120, ...base }, (w) =>
          uplinked || !w.nodes.some((n) => n.id === 'home-isp') ? w : { ...w, links: [...w.links, linkBetween(id, 'home-isp', 'fibre', w.links.map((l) => l.id))] },
        );
      }
      case 'switch':
      case 'access_point':
        return place({ kind: 'switch', ...base });
      case 'ups':
      case 'inverter':
      case 'dc_ups':
        return place({ kind: 'power', ...base });
      case 'drive':
      case 'enclosure':
      case 'usb_nic':
      case 'usb_hub': {
        const machine = world.nodes.find((n) => n.id === selected);
        if (!isHost(machine) || machine.runs_on || machine.role === 'client') {
          setNotice('Select a machine on the board first, then add this part to it.');
          return;
        }
        edit((w) => ({
          ...w,
          nodes: w.nodes.map((n) => (n.id === machine.id && isHost(n) ? { ...n, parts: [...(n.parts ?? []), { id: uniqueId(pack.id, (n.parts ?? []).map((p) => p.id)), pack: pack.id }] } : n)),
        }));
        setNotice(`Added to ${machine.label}.`);
        setTab('inspect');
      }
    }
  };

  const select = (node: Id | null) => {
    if (connecting && node && node !== connecting) {
      const choices = cableChoices(world, connecting, node);
      const [only] = choices;
      if (choices.length === 0) setNotice('Those two cannot be connected to each other. Computers and clients connect to a router or a switch.');
      else if (choices.length === 1 && only) edit((w) => ({ ...w, links: [...w.links, linkBetween(connecting, node, only, w.links.map((l) => l.id))] }));
      else setCable({ a: connecting, b: node, choices });
      setConnecting(null);
      return;
    }
    setConnecting(null);
    setSelected(node);
    if (node) setTab('inspect');
  };

  const remove = (node: Id) => {
    setSelected(null);
    setTab('report');
    setBuild((b) => {
      if (!b) return b;
      const wide = { ...b.layout.wide };
      delete wide[node];
      return { ...b, world: removeNode(b.world, node), layout: { ...b.layout, wide } };
    });
  };

  const exportFile = () => {
    const url = URL.createObjectURL(new Blob([toEnvelope(build, prices)], { type: 'application/json' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = `${build.name.replace(/[^\w-]+/g, '-').toLowerCase() || 'build'}.json`;
    link.click();
    URL.revokeObjectURL(url);
  };
  const copyLink = async () => {
    const payload = await sharePayload(build, prices);
    if (payload.length > SHARE_LIMIT) {
      setShared('This build is too large for a link. Export it to a file instead.');
      return;
    }
    const link = `${location.origin}${location.pathname}${href.buildImport(payload)}`;
    try {
      await navigator.clipboard.writeText(link);
      setShared('Link copied. The build is inside the link itself and is not sent to any server.');
    } catch {
      setShared(link);
    }
  };
  const label = (node: Id) => world.nodes.find((n) => n.id === node)?.label ?? node;
  const own = (world.custom_devices ?? []) as DevicePack[];

  return (
    <div className="build" data-testid="build">
      <header className="build__bar">
        <a className="topbar__back" href={href.builds} aria-label="Back to your builds">
          <ArrowLeftIcon size={18} />
        </a>
        <input className="input build__name" aria-label="Name of this build" value={build.name} maxLength={80} data-testid="build-name" onChange={(event) => setBuild({ ...build, name: event.target.value })} />
        <label className="check">
          <input type="checkbox" checked={showPower} onChange={(event) => setShowPower(event.target.checked)} />
          <span>Show power</span>
        </label>
        <label className="check">
          <input type="checkbox" checked={prices} data-testid="include-prices" onChange={(event) => setPrices(event.target.checked)} />
          <span>Include my prices</span>
        </label>
        <button type="button" className="btn" data-testid="copy-link" onClick={() => void copyLink()}>
          Copy link
        </button>
        <button type="button" className="btn" data-testid="export" onClick={exportFile}>
          Export file
        </button>
      </header>
      <StorageNotice />
      {shared && (
        <p className="notice" role="status" data-testid="share-status">
          {shared}
        </p>
      )}
      {connecting && (
        <p className="notice" role="status" data-testid="connect-hint">
          Now choose what to connect {label(connecting)} to.{' '}
          <button type="button" className="linkbtn" onClick={() => setConnecting(null)}>
            Cancel
          </button>
        </p>
      )}
      {cable && (
        <p className="notice" role="group" aria-label="Choose a cable" data-testid="cable-picker">
          Connect {label(cable.a)} to {label(cable.b)} by:{' '}
          {cable.choices.map((medium) => (
            <button key={medium} type="button" className="btn" data-testid={`cable-${medium}`} onClick={() => { edit((w) => ({ ...w, links: [...w.links, linkBetween(cable.a, cable.b, medium, w.links.map((l) => l.id))] })); setCable(null); }}>
              {medium === 'wifi' ? 'Wi-Fi' : 'Ethernet cable'}
            </button>
          ))}
          <button type="button" className="linkbtn" onClick={() => setCable(null)}>
            Cancel
          </button>
        </p>
      )}
      <div className="build__cols">
        <Palette
          catalog={catalog}
          own={own}
          notice={notice}
          onAdd={add}
          onCustom={(pack) => {
            edit((w) => ({ ...w, custom_devices: [...(w.custom_devices ?? []), pack] }));
            place({ kind: 'host', id: uniqueId(pack.label, world.nodes.map((n) => n.id)), label: pack.label, device: pack.id });
          }}
        />
        <Board world={world} layout={layout} report={result.report} selected={selected} connecting={connecting} showPower={showPower} onSelect={select} onMove={move} />
        <aside className="build__side">
          <div className="btabs" role="tablist" aria-label="Build panels">
            {(['report', 'goals', 'inspect'] as const).map((name) => (
              <button key={name} type="button" role="tab" aria-selected={tab === name} className="btabs__tab" data-testid={`tab-${name}`} disabled={name === 'inspect' && !selected} onClick={() => setTab(name)}>
                {name === 'report' ? 'Report' : name === 'goals' ? 'Goals' : 'Selected part'}
              </button>
            ))}
          </div>
          <div className="build__panel" role="tabpanel">
            {result.error && <p role="alert">The simulator could not read this build: {result.error}</p>}
            {tab === 'report' && result.report && <ReportPanel report={result.report} world={world} catalog={catalog} bundle={bundle} />}
            {tab === 'goals' && <Goals world={world} catalog={catalog} edit={edit} />}
            {tab === 'inspect' && selected && <Inspector world={world} catalog={catalog} report={result.report} id={selected} edit={edit} onConnect={(node) => { setConnecting(node); setNotice(null); }} onDelete={remove} />}
          </div>
        </aside>
      </div>
    </div>
  );
}
