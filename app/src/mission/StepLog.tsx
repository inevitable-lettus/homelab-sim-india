// The step log is the canonical record of a trace, and its accessible form.
// Every line is the core's own step, in order, with the core's own numbers.
import { useEffect, useRef, useSyncExternalStore } from 'react';
import { size, time } from '../lib/format';
import type { TimelinePlayer } from '../lib/player';
import type { LegSpan } from '../lib/timeline';
import type { Bundle, HopTiming, Id, Step, World } from '../lib/types';
import { tableName, tableSummary } from './TableCallout';

interface Props {
  player: TimelinePlayer;
  world: World;
  bundle: Bundle;
}

function legTitle(leg: LegSpan, label: (id: Id) => string): string {
  if (leg.role === 'setup') return `First, ${label(leg.from)} dials out`;
  if (leg.role === 'reply') return 'The reply';
  return 'The request';
}

const PARTS: [keyof HopTiming, string, string][] = [
  ['propagation_ns', 'distance', 'prop'],
  ['serialization_ns', 'sending the bits', 'ser'],
  ['queue_ns', 'waiting in the queue', 'queue'],
  ['extra_ns', 'radio scheduling', 'extra'],
  ['processing_ns', 'handling at this box', 'proc'],
];

/** Where one hop's time went. All five numbers are the core's. */
function TimingBar({ timing }: { timing: HopTiming }) {
  const total = timing.total_ns;
  if (total === null || total === 0) return <p className="steps__timing-note">We don't have the data to time this hop.</p>;
  const parts = PARTS.map(([key, label, tone]) => ({ label, tone, ns: (timing[key] as number | null) ?? 0 })).filter((p) => p.ns > 0);
  return (
    <div className="timing">
      <div className="timing__bar" aria-hidden="true">
        {parts.map((part) => (
          <span key={part.tone} data-part={part.tone} style={{ flexGrow: Math.max(part.ns / total, 0.012) }} />
        ))}
      </div>
      <ul className="timing__legend">
        {parts.map((part) => (
          <li key={part.tone}>
            <span className="timing__key" data-part={part.tone} aria-hidden="true" />
            {time(part.ns)} {part.label}
          </li>
        ))}
      </ul>
      <p className="steps__timing-note">
        {size(timing.bytes)} on the wire. Estimate from this model's distances and link speeds.
      </p>
    </div>
  );
}

function StepDetail({ step, kindOf }: { step: Step; kindOf: (id: Id) => World['nodes'][number]['kind'] | undefined }) {
  const table = tableSummary(step);
  const row = step.table?.row;
  return (
    <div className="steps__detail">
      {step.before && (step.before.src !== step.packet.src || step.before.dst !== step.packet.dst) && (
        <dl className="swap">
          <dt>Before</dt>
          <dd>
            {step.before.src} to {step.before.dst}
          </dd>
          <dt>After</dt>
          <dd>
            {step.packet.src} to {step.packet.dst}
          </dd>
        </dl>
      )}
      {step.packet.inner && (
        <p className="steps__inner">
          Carrying, encrypted: {step.packet.inner.src} to {step.packet.inner.dst}
        </p>
      )}
      {table && (
        <p className="steps__table" data-tone={table.tone}>
          <strong>{tableName(kindOf(step.at))}:</strong> {table.head}
          {row && (
            <>
              {' '}
              <span className="mono">
                {row.inside} ⇄ {row.outside}
              </span>
            </>
          )}
        </p>
      )}
      {step.timing && <TimingBar timing={step.timing} />}
    </div>
  );
}

export function StepLog({ player, world, bundle }: Props) {
  const state = useSyncExternalStore(player.subscribe, player.getState);
  const timeline = player.getTimeline();
  const label = (id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id;
  const kindOf = (id: Id) => world.nodes.find((n) => n.id === id)?.kind;
  const current = useRef<HTMLLIElement>(null);
  const list = useRef<HTMLDivElement>(null);

  // Keep the current step in view inside the log, without moving the page.
  useEffect(() => {
    const item = current.current;
    const box = list.current;
    if (!item || !box) return;
    const top = item.offsetTop - box.offsetTop;
    if (top < box.scrollTop || top + item.offsetHeight > box.scrollTop + box.clientHeight) {
      box.scrollTop = Math.max(0, top - 24);
    }
  }, [state.step]);

  if (timeline.steps.length === 0) {
    return (
      <div className="steps steps--empty" ref={list}>
        <p>No packet leaves the device for this attempt. The result above says why.</p>
      </div>
    );
  }
  if (state.step < 0) {
    return (
      <div className="steps steps--empty" ref={list}>
        <p>Press Run. Each hop the packet makes is listed here, with what changed and how long it took.</p>
      </div>
    );
  }

  const shown = state.ended ? timeline.steps.length - 1 : state.step;
  return (
    <div className="steps" ref={list} role="log" aria-label="Step log" data-testid="step-log">
      {timeline.legs
        .filter((leg) => leg.firstStep <= shown)
        .map((leg) => (
          <section key={leg.id} className="steps__leg">
            <h4>
              {legTitle(leg, label)}
              {leg.durationNs !== null && leg.lastStep <= shown && <span className="steps__time">{time(leg.durationNs)} one way</span>}
            </h4>
            <ol start={leg.firstStep + 1}>
              {timeline.steps
                .filter((flat) => flat.index >= leg.firstStep && flat.index <= Math.min(leg.lastStep, shown))
                .map(({ index, step }) => {
                  const words = bundle.explanations[step.reason];
                  const active = index === state.step;
                  return (
                    <li
                      key={index}
                      ref={active ? current : undefined}
                      className="steps__item"
                      data-active={active || undefined}
                      data-action={step.action}
                      data-testid="step"
                      data-reason={step.reason}
                      data-at={step.at}
                    >
                      <button type="button" className="steps__head" onClick={() => player.showStep(index)} aria-current={active ? 'step' : undefined}>
                        <span className="steps__n">{index + 1}</span>
                        <span className="steps__what">
                          <span className="steps__where">{label(step.at)}</span>
                          <span className="steps__title">{words?.title ?? step.reason}</span>
                        </span>
                        {step.timing?.arrive_ns != null && <span className="steps__time">+{time(step.timing.arrive_ns)}</span>}
                      </button>
                      <p className="steps__line">{words?.line}</p>
                      <p className="steps__packet">
                        {step.packet.src} <span aria-hidden="true">→</span>
                        <span className="sr-only"> to </span> {step.packet.dst}
                      </p>
                      {active && <StepDetail step={step} kindOf={kindOf} />}
                    </li>
                  );
                })}
            </ol>
          </section>
        ))}
    </div>
  );
}
