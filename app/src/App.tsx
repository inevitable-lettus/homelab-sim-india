import { useEffect, useState } from 'react';
import { useProgress, useReducedMotion } from './lib/progress';
import { useRoute } from './lib/route';
import { loadSim } from './lib/sim';
import type { Bundle } from './lib/types';
import { Home } from './screens/Home';
import { MissionScreen } from './screens/MissionScreen';
import { Concept, Credits, Settings } from './screens/Pages';

type Engine = { kind: 'loading' } | { kind: 'ready'; bundle: Bundle } | { kind: 'failed'; message: string };

export function App() {
  const [engine, setEngine] = useState<Engine>({ kind: 'loading' });
  const route = useRoute();
  const { theme } = useProgress();
  const reduced = useReducedMotion();

  useEffect(() => {
    let active = true;
    loadSim().then(
      (bundle) => active && setEngine({ kind: 'ready', bundle }),
      (error: unknown) => active && setEngine({ kind: 'failed', message: error instanceof Error ? error.message : String(error) }),
    );
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'system') delete root.dataset.theme;
    else root.dataset.theme = theme;
    root.dataset.motion = reduced ? 'reduced' : 'full';
  }, [theme, reduced]);

  const bundle = engine.kind === 'ready' ? engine.bundle : null;
  useEffect(() => {
    const mission = route.name === 'mission' ? bundle?.missions[route.id] : undefined;
    const concept = route.name === 'concept' ? bundle?.explanations[route.id] : undefined;
    const part = mission?.title ?? concept?.title ?? (route.name === 'settings' ? 'Settings' : route.name === 'credits' ? 'Source and credits' : '');
    document.title = part ? `${part} · Homelab Sim` : 'Homelab Sim: remote access on Indian broadband';
    window.scrollTo(0, 0);
  }, [route, bundle]);

  if (engine.kind === 'loading') {
    return (
      <div className="boot" role="status" data-testid="engine-status">
        <span className="boot__bar" />
        <span className="boot__bar boot__bar--short" />
        <span className="sr-only">Starting the simulator</span>
      </div>
    );
  }
  if (engine.kind === 'failed') {
    return (
      <div className="page" role="alert" data-testid="engine-status">
        <h1>The simulator did not start</h1>
        <p>This needs a browser with WebAssembly. Reloading the page often helps if the download was cut short.</p>
        <p className="mono">{engine.message}</p>
        <button type="button" className="btn btn--primary" onClick={() => location.reload()}>
          Reload
        </button>
      </div>
    );
  }

  switch (route.name) {
    case 'mission':
      return <MissionScreen bundle={engine.bundle} id={route.id} />;
    case 'concept':
      return <Concept bundle={engine.bundle} id={route.id} />;
    case 'settings':
      return <Settings />;
    case 'credits':
      return <Credits />;
    case 'home':
      return <Home bundle={engine.bundle} />;
  }
}
