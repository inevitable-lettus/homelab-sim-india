// The files a terminal mission grades: an editor for each, a key tool, and
// the core's verdict on what was last checked. The editor works everywhere.
// The real terminal is an extra, offered below it.
import { useId, useState, type ReactNode } from 'react';
import { StatusMark } from '../components/Status';
import { newKeyPair, withPrivateKey, type KeyPair } from '../lib/keys';
import type { ArtifactResult, ArtifactSpec, Bundle, Depth } from '../lib/types';

export type FileTexts = Record<string, string>;

/** What each file holds before the learner has touched it. */
export function starterFiles(artifacts: ArtifactSpec[], depth: Depth): FileTexts {
  return Object.fromEntries(artifacts.map((spec) => [spec.path, depth === 'guided' ? spec.starter : '']));
}

interface FilesProps {
  artifacts: ArtifactSpec[];
  depth: Depth;
  files: FileTexts;
  /** The core's results for the files as last checked, or null before any check. */
  results: ArtifactResult[] | null;
  /** True when the files have changed since they were checked. */
  dirty: boolean;
  /** Set while the terminal owns the files, so the editor shows a copy. */
  locked: boolean;
  busy: boolean;
  bundle: Bundle;
  onEdit: (path: string, text: string) => void;
  onCheck: () => void;
  onStartAgain: () => void;
  terminal?: ReactNode;
}

function KeyTool({ artifacts, depth, files, onEdit, locked }: Pick<FilesProps, 'artifacts' | 'depth' | 'files' | 'onEdit' | 'locked'>) {
  const [pair, setPair] = useState<KeyPair | null>(null);
  const [copied, setCopied] = useState('');
  const conf = artifacts.find((spec) => spec.kind === 'wireguard_conf');
  const keyFile = artifacts.find((spec) => spec.kind === 'wireguard_public_key');

  const copy = (what: string, text: string) => {
    void navigator.clipboard?.writeText(text).then(
      () => setCopied(what),
      () => setCopied(''),
    );
  };
  const place = () => {
    if (!pair) return;
    if (conf) onEdit(conf.path, withPrivateKey(files[conf.path] ?? '', pair.privateKey));
    if (keyFile) onEdit(keyFile.path, `${pair.publicKey}\n`);
  };

  return (
    <div className="keytool">
      <h4>Key pair</h4>
      <p className="files__note">
        The same as <code>wg genkey</code> and <code>wg pubkey</code>. Made in this browser, for this lesson only. Do not use these keys on a real server.
      </p>
      <button type="button" className="btn" onClick={() => setPair(newKeyPair())} data-testid="make-keys">
        {pair ? 'Make another pair' : 'Make a key pair'}
      </button>
      {pair && (
        <dl className="keytool__keys" aria-live="polite">
          <dt>Private key (stays on the server)</dt>
          <dd>
            <code data-testid="private-key">{pair.privateKey}</code>
            <button type="button" className="link-btn" onClick={() => copy('private', pair.privateKey)}>
              {copied === 'private' ? 'Copied' : 'Copy'}
            </button>
          </dd>
          <dt>Public key (give this to the other end)</dt>
          <dd>
            <code data-testid="public-key">{pair.publicKey}</code>
            <button type="button" className="link-btn" onClick={() => copy('public', pair.publicKey)}>
              {copied === 'public' ? 'Copied' : 'Copy'}
            </button>
          </dd>
        </dl>
      )}
      {pair && depth === 'guided' && !locked && (
        <button type="button" className="btn" onClick={place} data-testid="place-keys">
          Put this pair in my files
        </button>
      )}
    </div>
  );
}

export function Files(props: FilesProps) {
  const { artifacts, files, results, dirty, locked, busy, bundle, onEdit, onCheck, onStartAgain, terminal } = props;
  const id = useId();
  return (
    <section className="files" aria-labelledby={`${id}-title`}>
      <h3 id={`${id}-title`}>Your files</h3>
      {locked && <p className="files__note">The terminal is open, so these are the files inside it. Edit them there, then check.</p>}
      {artifacts.map((spec) => {
        const result = !dirty ? results?.find((r) => r.id === spec.id) : undefined;
        const words = result ? bundle.explanations[result.reason] : undefined;
        const single = spec.kind === 'wireguard_public_key';
        return (
          <div key={spec.id} className="file" data-status={result?.status ?? 'waiting'} data-testid={`file-${spec.id}`}>
            <div className="file__head">
              <label htmlFor={`${id}-${spec.id}`}>
                <span className="file__label">{spec.label}</span>
                <code className="file__path">{spec.path}</code>
              </label>
              <StatusMark status={result?.status ?? 'waiting'} word={result ? (words?.title ?? result.reason) : results && dirty ? 'Changed since the last check' : 'Not checked yet'} />
            </div>
            <textarea
              id={`${id}-${spec.id}`}
              className="input mono file__editor"
              rows={single ? 2 : 14}
              value={files[spec.path] ?? ''}
              readOnly={locked}
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              autoComplete="off"
              aria-describedby={result ? `${id}-${spec.id}-result` : undefined}
              aria-invalid={result?.status === 'fail' || undefined}
              onChange={(event) => onEdit(spec.path, event.target.value)}
              data-testid={`editor-${spec.id}`}
            />
            {result && result.status !== 'pass' && (
              <div className="file__result" id={`${id}-${spec.id}-result`}>
                <p>{words?.line}</p>
                {result.problems.length > 0 && (
                  <ul className="file__problems" data-testid={`problems-${spec.id}`}>
                    {result.problems.map((problem, i) => (
                      <li key={i}>
                        {problem.line ? <span className="file__line">Line {problem.line}</span> : null}
                        {problem.message}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>
        );
      })}
      <div className="files__actions">
        <button type="button" className="btn btn--primary" onClick={onCheck} disabled={busy} data-testid="check-files">
          {busy ? 'Reading the files' : 'Check my files'}
        </button>
        {!locked && (
          <button type="button" className="link-btn" onClick={onStartAgain}>
            Start the files again
          </button>
        )}
      </div>
      <p className="files__note">Checking reads the files. Then press Run to see what the network does with them.</p>
      <KeyTool {...props} />
      {terminal}
    </section>
  );
}
