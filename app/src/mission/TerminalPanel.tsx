// The offer of a real terminal, and the terminal once it is running. Nothing
// is downloaded until the learner presses the button, and the size is shown
// first.
import { Suspense, lazy, useEffect, useState } from 'react';
import { href } from '../lib/route';
import { bootGuest, loadGuestManifest, type GuestManifest, type TerminalHost } from '../lib/terminal';

const TerminalView = lazy(() => import('./TerminalView'));

type Phase = { name: 'offer' } | { name: 'starting'; fraction: number } | { name: 'running' } | { name: 'failed' };

interface Props {
  /** The files to put in the guest when it starts, by path. */
  files: Record<string, string>;
  host: TerminalHost | null;
  onHost: (host: TerminalHost | null) => void;
}

const megabytes = (bytes: number) => Math.ceil(bytes / 1_000_000);

export function TerminalPanel({ files, host, onHost }: Props) {
  const [guest, setGuest] = useState<GuestManifest | null | undefined>(undefined);
  const [state, setPhase] = useState<Phase>({ name: 'offer' });
  // The guest outlives this panel when the layout moves it.
  const phase: Phase = host ? { name: 'running' } : state.name === 'running' ? { name: 'offer' } : state;
  useEffect(() => {
    let live = true;
    void loadGuestManifest().then((found) => live && setGuest(found));
    return () => {
      live = false;
    };
  }, []);

  if (guest === undefined) return null;
  if (guest === null) {
    return (
      <div className="terminal">
        <h4>Real terminal</h4>
        <p className="files__note">This copy of the site was built without the Linux image, so only the editor is available here.</p>
      </div>
    );
  }

  const start = async () => {
    setPhase({ name: 'starting', fraction: 0 });
    try {
      const booted = await bootGuest(guest, (fraction) => setPhase({ name: 'starting', fraction }));
      const encoder = new TextEncoder();
      for (const [path, text] of Object.entries(files)) {
        if (text) await booted.writeFile(path, encoder.encode(text));
      }
      onHost(booted);
      setPhase({ name: 'running' });
    } catch {
      setPhase({ name: 'failed' });
    }
  };
  const stop = () => {
    onHost(null);
    setPhase({ name: 'offer' });
  };

  return (
    <div className="terminal" data-phase={phase.name} data-testid="terminal">
      <h4>Real terminal</h4>
      {phase.name === 'offer' && (
        <>
          <p className="files__note">
            A small Linux system (Alpine {guest.alpine}) that runs inside this page, with <code>wg</code>, <code>nano</code> and <code>vi</code>. It cannot reach the internet. It downloads up to{' '}
            {megabytes(guest.first_download_bytes)} MB and needs about 500 MB of memory, so on a phone the editor above is the better choice.
          </p>
          <button type="button" className="btn" onClick={() => void start()} data-testid="terminal-start">
            Open the terminal (up to {megabytes(guest.first_download_bytes)} MB)
          </button>
        </>
      )}
      {phase.name === 'starting' && (
        <p className="files__note" role="status">
          Starting Linux. Downloaded {Math.round(phase.fraction * 100)}%.
        </p>
      )}
      {phase.name === 'failed' && (
        <>
          <p className="files__note" role="status">
            The terminal could not start in this browser. The editor above does the same job.
          </p>
          <button type="button" className="btn" onClick={() => void start()}>
            Try again
          </button>
        </>
      )}
      {phase.name === 'running' && host && (
        <>
          <p className="files__note">
            Your files are in <code>/root</code>. Edit them here, for example with <code>nano wg0.conf</code>, then press Check my files.
          </p>
          <Suspense fallback={<p className="files__note">Loading the terminal screen.</p>}>
            <TerminalView host={host} />
          </Suspense>
          <div className="files__actions">
            <button type="button" className="link-btn" onClick={stop} data-testid="terminal-stop">
              Close the terminal
            </button>
            <a className="link" href={href.credits}>
              What is in this Linux image
            </a>
          </div>
        </>
      )}
    </div>
  );
}
