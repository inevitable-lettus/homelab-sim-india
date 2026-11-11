// The quieter pages: one concept, settings, credits.
import { useId, useRef, useState, type ReactNode } from 'react';
import { ExplainSections } from '../components/Explain';
import { ArrowLeftIcon } from '../components/icons';
import { progress, useProgress, type MotionSetting, type ThemeSetting } from '../lib/progress';
import { href } from '../lib/route';
import { sourceLink } from '../lib/source';
import type { Bundle, Depth } from '../lib/types';

function Page({ title, children, back = href.home, backLabel = 'Track' }: { title: string; children: ReactNode; back?: string; backLabel?: string }) {
  return (
    <div className="page">
      <a className="page__back" href={back}>
        <ArrowLeftIcon size={18} />
        <span>{backLabel}</span>
      </a>
      <main>
        <h1>{title}</h1>
        {children}
      </main>
    </div>
  );
}

export function Concept({ bundle, id }: { bundle: Bundle; id: string }) {
  const explanation = bundle.explanations[id];
  if (!explanation) {
    return (
      <Page title="No such page">
        <p>There is no explanation with that name.</p>
      </Page>
    );
  }
  const uses = Object.values(bundle.missions)
    .filter((m) => m.debrief.concept === id)
    .sort((a, b) => a.order - b.order);
  return (
    <Page title={explanation.title}>
      <p className="page__lede">{explanation.line}</p>
      <ExplainSections explanation={explanation} />
      {uses.length > 0 && (
        <p className="page__uses">
          Taught in{' '}
          {uses.map((m, i) => (
            <span key={m.id}>
              {i > 0 && ', '}
              <a className="link" href={href.mission(m.id)}>
                Mission {m.order}: {m.title}
              </a>
            </span>
          ))}
          .
        </p>
      )}
    </Page>
  );
}

function Choice<T extends string>({ legend, help, value, options, onChange }: { legend: string; help?: string; value: T; options: [T, string][]; onChange: (value: T) => void }) {
  const name = useId();
  return (
    <fieldset className="control setting">
      <legend>{legend}</legend>
      {help && <p className="setting__help">{help}</p>}
      <div className="choices">
        {options.map(([option, label]) => (
          <label key={option} className="choice">
            <input type="radio" name={name} checked={value === option} onChange={() => onChange(option)} />
            <span className="choice__text">{label}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}

export function Settings() {
  const saved = useProgress();
  const file = useRef<HTMLInputElement>(null);
  const [note, setNote] = useState('');
  const [confirming, setConfirming] = useState(false);
  const count = Object.keys(saved.done).length;

  const download = () => {
    const url = URL.createObjectURL(new Blob([progress.export()], { type: 'application/json' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = 'homelab-sim-progress.json';
    link.click();
    URL.revokeObjectURL(url);
  };

  return (
    <Page title="Settings">
      <Choice<Depth>
        legend="Depth"
        help="Guided gives a suggested approach, hints step by step, and explanations when something fails. Challenge gives the goal and one hint."
        value={saved.depth}
        options={[
          ['guided', 'Guided'],
          ['challenge', 'Challenge'],
        ]}
        onChange={(depth) => progress.update({ depth })}
      />
      <Choice<MotionSetting>
        legend="Motion"
        help="With reduced motion, packets do not travel. You step through a trace by hand and each change is shown as a before and after."
        value={saved.motion}
        options={[
          ['system', 'Follow my device'],
          ['reduced', 'Reduced'],
          ['full', 'Full'],
        ]}
        onChange={(motion) => progress.update({ motion })}
      />
      <Choice<ThemeSetting>
        legend="Theme"
        value={saved.theme}
        options={[
          ['system', 'Follow my device'],
          ['light', 'Light'],
          ['dark', 'Dark'],
        ]}
        onChange={(theme) => progress.update({ theme })}
      />

      <section className="setting">
        <h2>Your progress</h2>
        <p className="setting__help">
          {count === 0 ? 'No missions finished yet.' : `${count} mission${count === 1 ? '' : 's'} finished.`} Progress is kept in this browser only. It is never sent anywhere. Export it to move to another device.
        </p>
        <div className="setting__row">
          <button type="button" className="btn" onClick={download}>
            Export progress
          </button>
          <button type="button" className="btn" onClick={() => file.current?.click()}>
            Import progress
          </button>
          <input
            ref={file}
            type="file"
            accept="application/json,.json"
            hidden
            onChange={async (event) => {
              const picked = event.target.files?.[0];
              if (!picked) return;
              setNote(progress.import(await picked.text()) ? 'Progress imported.' : 'That file is not a Homelab Sim progress file. Nothing was changed.');
              event.target.value = '';
            }}
          />
          {confirming ? (
            <>
              <button
                type="button"
                className="btn btn--danger"
                onClick={() => {
                  progress.clear();
                  setConfirming(false);
                  setNote('Everything was cleared.');
                }}
              >
                Yes, clear everything
              </button>
              <button type="button" className="btn" onClick={() => setConfirming(false)}>
                Keep it
              </button>
            </>
          ) : (
            <button type="button" className="btn" onClick={() => setConfirming(true)}>
              Clear all data
            </button>
          )}
        </div>
        <p className="setting__note" role="status">
          {note}
        </p>
      </section>
    </Page>
  );
}

const THIRD_PARTY: [string, string, string][] = [
  ['React', 'MIT', 'https://react.dev'],
  ['React Flow (@xyflow/react)', 'MIT', 'https://reactflow.dev'],
  ['wasm-bindgen', 'MIT or Apache-2.0', 'https://github.com/rustwasm/wasm-bindgen'],
  ['serde, serde_json', 'MIT or Apache-2.0', 'https://serde.rs'],
];

export function Credits() {
  return (
    <Page title="Source and credits">
      <div className="prose">
        <p>
          Homelab Sim is free software. The code is licensed under the GNU Affero General Public License, version 3. The lessons, mission files and data are licensed under Creative Commons Attribution-ShareAlike 4.0.
        </p>
        <p>
          {sourceLink ? (
            <>
              The exact source of the version you are using: <a href={sourceLink.href}>{sourceLink.label}</a>.
            </>
          ) : (
            'This is a local build, so it carries no source link. The published site links to the exact commit it was built from.'
          )}
        </p>
        <h2>What it does not do</h2>
        <ul>
          <li>It has no server, no accounts and no analytics.</li>
          <li>It never asks for, reads or sends your IP address.</li>
          <li>Your progress stays in this browser.</li>
          <li>The simulator is a deterministic program. No AI model runs when you play.</li>
        </ul>
        <h2>About the networks you see</h2>
        <p>
          Every address, distance and speed in the missions is a teaching model. The addresses come from ranges reserved for documentation. The ISP in the missions is invented and says nothing about any real provider.
        </p>
        <h2>Built with</h2>
        <ul>
          {THIRD_PARTY.map(([name, licence, url]) => (
            <li key={name}>
              <a href={url} target="_blank" rel="noreferrer noopener">
                {name}
              </a>
              , {licence}
            </li>
          ))}
        </ul>
      </div>
    </Page>
  );
}
