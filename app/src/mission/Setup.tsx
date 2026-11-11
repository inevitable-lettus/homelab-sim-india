// The things a mission lets the learner change. Each control only records a
// choice in the world. The core works out what the choice does.
import { useId, useState } from 'react';
import { PlusIcon, TrashIcon } from '../components/icons';
import { resolveTarget } from '../lib/sim';
import type { Control, Id, Mission, NodeFacts, PortForward, Proto, Question, World } from '../lib/types';
import {
  answerOf,
  forwardsOf,
  isJoined,
  looksLikeIpv4,
  parsePort,
  removeForward,
  sameTarget,
  saveForward,
  setAnswer,
  setDial,
  setJoined,
} from '../lib/world';

interface Common {
  world: World;
  onChange: (world: World) => void;
}

const labelOf = (world: World, id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id;

export function DialControl({ control, world, onChange }: Common & { control: Extract<Control, { kind: 'dial' }> }) {
  const attempt = world.attempts.find((a) => a.id === control.attempt);
  const name = useId();
  if (!attempt) return null;
  return (
    <fieldset className="control">
      <legend>What {labelOf(world, attempt.from).toLowerCase()} dials</legend>
      <div className="choices">
        {control.options.map((option) => {
          const resolved = resolveTarget(world, option.target);
          return (
            <label key={option.label} className="choice">
              <input
                type="radio"
                name={name}
                checked={sameTarget(attempt.to, option.target)}
                onChange={() => onChange(setDial(world, attempt.id, option.target))}
              />
              <span className="choice__text">
                <span>{option.label}</span>
                <span className="choice__value">
                  {resolved.address === null ? 'no address yet' : resolved.address === 'unknown' ? 'unknown' : `${resolved.address}:${attempt.port}`}
                </span>
              </span>
            </label>
          );
        })}
      </div>
    </fieldset>
  );
}

interface Draft {
  id: Id;
  external: string;
  ip: string;
  internal: string;
  proto: Proto;
}

function ForwardForm({ draft, onSave, onCancel }: { draft: Draft; onSave: (forward: PortForward) => void; onCancel: () => void }) {
  const [form, setForm] = useState(draft);
  const [tried, setTried] = useState(false);
  const id = useId();
  const external = parsePort(form.external);
  const internal = parsePort(form.internal);
  const ipOk = looksLikeIpv4(form.ip);
  const errors = {
    external: external === null ? 'A port is a number from 1 to 65535.' : '',
    ip: ipOk ? '' : 'Write a home address, such as 192.168.1.20.',
    internal: internal === null ? 'A port is a number from 1 to 65535.' : '',
  };
  const field = (key: 'external' | 'ip' | 'internal', label: string, hint: string, mode: 'numeric' | 'decimal') => (
    <div className="field">
      <label htmlFor={`${id}-${key}`}>{label}</label>
      <input
        id={`${id}-${key}`}
        className="input mono"
        inputMode={mode}
        autoComplete="off"
        spellCheck={false}
        value={form[key]}
        placeholder={hint}
        aria-invalid={tried && !!errors[key]}
        aria-describedby={tried && errors[key] ? `${id}-${key}-error` : undefined}
        onChange={(event) => setForm({ ...form, [key]: event.target.value })}
      />
      {tried && errors[key] && (
        <p className="field__error" id={`${id}-${key}-error`}>
          {errors[key]}
        </p>
      )}
    </div>
  );
  return (
    <form
      className="forward-form"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        setTried(true);
        if (external === null || internal === null || !ipOk) return;
        onSave({ id: form.id, enabled: true, proto: form.proto, external_port: external, to_ip: form.ip.trim(), internal_port: internal });
      }}
    >
      {field('external', 'Port on the router', '8096', 'numeric')}
      {field('ip', 'Send to this home address', '192.168.1.20', 'decimal')}
      {field('internal', 'Port on that device', '8096', 'numeric')}
      <div className="field">
        <label htmlFor={`${id}-proto`}>Protocol</label>
        <select id={`${id}-proto`} className="input" value={form.proto} onChange={(event) => setForm({ ...form, proto: event.target.value as Proto })}>
          <option value="tcp">TCP</option>
          <option value="udp">UDP</option>
        </select>
      </div>
      <div className="forward-form__actions">
        <button type="submit" className="btn btn--primary" data-testid="save-forward">
          Save forward
        </button>
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}

export function ForwardsControl({ routerId, world, onChange }: Common & { routerId: Id }) {
  const forwards = forwardsOf(world, routerId);
  const [editing, setEditing] = useState<Draft | null>(null);
  const blank = (): Draft => ({ id: `forward-${Date.now().toString(36)}`, external: '', ip: '', internal: '', proto: 'tcp' });
  return (
    <div className="control" role="group" aria-label={`Port forwards on ${labelOf(world, routerId)}`}>
      <p className="control__legend">Port forwards on {labelOf(world, routerId).toLowerCase()}</p>
      {forwards.length === 0 && !editing && <p className="control__empty">None yet. The router drops anything from outside that nothing inside asked for.</p>}
      <ul className="forwards">
        {forwards.map((forward) => (
          <li key={forward.id} className="forward" data-off={!forward.enabled || undefined}>
            <span className="forward__rule">
              <span className="forward__proto">{forward.proto.toUpperCase()}</span> {forward.external_port}
              <span aria-hidden="true"> → </span>
              <span className="sr-only"> to </span>
              {forward.to_ip}:{forward.internal_port}
            </span>
            <label className="switch">
              <input type="checkbox" checked={forward.enabled} onChange={(event) => onChange(saveForward(world, routerId, { ...forward, enabled: event.target.checked }))} />
              <span>{forward.enabled ? 'On' : 'Off'}</span>
            </label>
            <button
              type="button"
              className="icon-btn"
              aria-label={`Edit the forward for port ${forward.external_port}`}
              onClick={() => setEditing({ id: forward.id, external: String(forward.external_port), ip: forward.to_ip, internal: String(forward.internal_port), proto: forward.proto })}
            >
              <span className="icon-btn__text">Edit</span>
            </button>
            <button type="button" className="icon-btn" aria-label={`Remove the forward for port ${forward.external_port}`} onClick={() => onChange(removeForward(world, routerId, forward.id))}>
              <TrashIcon size={18} />
            </button>
          </li>
        ))}
      </ul>
      {editing ? (
        <ForwardForm
          key={editing.id}
          draft={editing}
          onCancel={() => setEditing(null)}
          onSave={(forward) => {
            onChange(saveForward(world, routerId, forward));
            setEditing(null);
          }}
        />
      ) : (
        <button type="button" className="btn" onClick={() => setEditing(blank())} data-testid="add-forward">
          <PlusIcon size={18} />
          <span>Add a port forward</span>
        </button>
      )}
    </div>
  );
}

export function JoinControl({ control, world, facts, onChange, only }: Common & { control: Extract<Control, { kind: 'overlay_join' }>; facts: NodeFacts[]; only?: Id }) {
  const overlay = world.overlays?.find((o) => o.id === control.network);
  return (
    <div className="control" role="group" aria-label={overlay?.label ?? 'Mesh VPN'}>
      <p className="control__legend">{overlay?.label ?? 'Mesh VPN'}</p>
      <ul className="joins">
        {control.nodes
          .filter((id) => !only || id === only)
          .map((id) => {
            const joined = isJoined(world, id, control.network);
            const address = facts.find((f) => f.id === id)?.tunnels?.find((t) => t.enabled)?.address;
            return (
              <li key={id}>
                <label className="switch switch--wide">
                  <input type="checkbox" checked={joined} onChange={(event) => onChange(setJoined(world, id, control.network, event.target.checked))} data-testid={`join-${id}`} />
                  <span>
                    {labelOf(world, id)}
                    <span className="switch__note">{joined ? `joined${address ? `, mesh address ${address}` : ''}` : 'not installed'}</span>
                  </span>
                </label>
              </li>
            );
          })}
      </ul>
    </div>
  );
}

export function QuestionControl({ question, world, onChange, disabled }: Common & { question: Question; disabled?: boolean }) {
  const name = useId();
  const chosen = answerOf(world, question.id);
  return (
    <fieldset className="control control--question" disabled={disabled}>
      <legend>{question.prompt}</legend>
      <div className="choices">
        {question.options.map((option) => (
          <label key={option.value} className="choice">
            <input type="radio" name={name} checked={chosen === option.value} onChange={() => onChange(setAnswer(world, question.id, option.value))} data-testid={`answer-${question.id}-${option.value}`} />
            <span className="choice__text">{option.label}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}

/** Which nodes carry a control, so the board can mark them. */
export function editableNodes(mission: Mission): Set<Id> {
  const ids = new Set<Id>();
  for (const control of mission.controls) {
    if (control.kind === 'dial') {
      const from = mission.world.attempts.find((a) => a.id === control.attempt)?.from;
      if (from) ids.add(from);
    } else if (control.kind === 'port_forwards') ids.add(control.router);
    else if (control.kind === 'overlay_join') control.nodes.forEach((id) => ids.add(id));
    else if (control.kind === 'tunnel') ids.add(control.node);
  }
  return ids;
}

/** The controls that belong to one node, or all of them. */
export function Controls({ mission, world, facts, onChange, node }: Common & { mission: Mission; facts: NodeFacts[]; node?: Id }) {
  const items = mission.controls.flatMap((control, i) => {
    if (control.kind === 'dial') {
      const from = world.attempts.find((a) => a.id === control.attempt)?.from;
      return !node || node === from ? [<DialControl key={i} control={control} world={world} onChange={onChange} />] : [];
    }
    if (control.kind === 'port_forwards') {
      return !node || node === control.router ? [<ForwardsControl key={i} routerId={control.router} world={world} onChange={onChange} />] : [];
    }
    if (control.kind === 'overlay_join') {
      return !node || control.nodes.includes(node) ? [<JoinControl key={i} control={control} world={world} facts={facts} onChange={onChange} only={node} />] : [];
    }
    return [];
  });
  return items.length ? <div className="controls">{items}</div> : null;
}
