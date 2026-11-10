import { useEffect, useState } from 'react';
import { runEngineCheck, type Report } from './lib/sim';
import { sourceLink } from './lib/source';

type EngineState =
  | { kind: 'running' }
  | { kind: 'ready'; report: Report }
  | { kind: 'failed'; message: string };

export function App() {
  const [engine, setEngine] = useState<EngineState>({ kind: 'running' });

  useEffect(() => {
    let active = true;
    runEngineCheck()
      .then((report) => {
        if (active) setEngine({ kind: 'ready', report });
      })
      .catch((error: unknown) => {
        if (active) setEngine({ kind: 'failed', message: String(error) });
      });
    return () => {
      active = false;
    };
  }, []);

  return (
    <div className="page">
      <main>
        <h1>Homelab Sim</h1>
        <p>Learn homelab networking on Indian ISP lines. Under construction.</p>
        <p data-testid="engine-status" role="status">
          <EngineMessage engine={engine} />
        </p>
      </main>
      <footer>
        <span>AGPL-3.0 code. CC BY-SA 4.0 content.</span>{' '}
        {sourceLink ? <a href={sourceLink.href}>{sourceLink.label}</a> : <span>Source not published yet.</span>}
      </footer>
    </div>
  );
}

function EngineMessage({ engine }: { engine: EngineState }) {
  if (engine.kind === 'running') return 'Starting engine…';
  if (engine.kind === 'failed') return `Engine failed: ${engine.message}`;
  const finding = engine.report.findings[0];
  return finding ? `Engine ready: ${finding.rule_id} is ${finding.status}.` : 'Engine ready: no findings.';
}
