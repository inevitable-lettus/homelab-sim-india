// What happened to the attempt, in words: where it ended and why. Every
// number is the core's and is labelled as an estimate.
import { ExplainSections, hasDetail } from '../components/Explain';
import { StatusMark } from '../components/Status';
import { long, rate, size, time } from '../lib/format';
import { href } from '../lib/route';
import type { Bundle, Id, Trace, World } from '../lib/types';

interface Props {
  trace: Trace;
  world: World;
  bundle: Bundle;
  /** Guided depth opens the explanation. Challenge keeps it shut until the mission is done. */
  explain: 'open' | 'closed' | 'locked';
}

export function Result({ trace, world, bundle, explain }: Props) {
  const label = (id: Id | null) => world.nodes.find((n) => n.id === id)?.label ?? 'the board';
  const words = trace.reason ? bundle.explanations[trace.reason] : undefined;
  const summary = trace.summary;
  const link = summary.transfer?.bottleneck_link ? world.links.find((l) => l.id === summary.transfer?.bottleneck_link) : undefined;
  const assumed = trace.assumed?.[0];

  return (
    <section className="result" data-outcome={trace.outcome} data-testid="result" aria-live="polite">
      {trace.outcome === 'delivered' && (
        <>
          <StatusMark status="pass" word="In this model" />
          <h3>Connected. {label(trace.ended_at)} answered, and the reply got back.</h3>
          <dl className="facts">
            <div>
              <dt>Round trip</dt>
              <dd>{summary.round_trip_ns === null ? 'unknown' : `about ${time(summary.round_trip_ns)}`}</dd>
            </div>
            <div>
              <dt>Hops out</dt>
              <dd>{summary.hops}</dd>
            </div>
            <div>
              <dt>Largest packet that fits</dt>
              <dd>{summary.path_mtu === null ? 'unknown' : `${summary.path_mtu} bytes`}</dd>
            </div>
            {summary.transfer && (
              <div className="facts__wide">
                <dt>A {size(summary.transfer.bytes)} film</dt>
                <dd>
                  {summary.transfer.duration_ms === null ? 'unknown' : `about ${long(summary.transfer.duration_ms)}`}
                  {link && summary.transfer.bottleneck_bps !== null && (
                    <span className="facts__note">
                      {' '}
                      The slowest link is {label(link.a)} to {label(link.b)}, with {rate(summary.transfer.bottleneck_bps)} to spare.
                    </span>
                  )}
                </dd>
              </div>
            )}
          </dl>
          <p className="result__estimate">Times are estimates from this model's distances and link speeds, not measurements.</p>
        </>
      )}
      {trace.outcome === 'dropped' && (
        <>
          <StatusMark status="fail" word={`Stopped at ${label(trace.ended_at)}`} />
          <h3>{words?.title ?? trace.reason}</h3>
          <p>{words?.line}</p>
        </>
      )}
      {trace.outcome === 'unknown' && (
        <>
          <StatusMark status="unknown" word="We don't have data for this" />
          <h3>{words?.title ?? 'We cannot tell'}</h3>
          <p>{words?.line}</p>
        </>
      )}
      {assumed && (
        <p className="result__estimate">
          We don't know this line's type. The trace shows a {assumed.ipv4 === 'cgnat' ? 'carrier-NAT' : 'public'} line. The result is the same either way.
        </p>
      )}
      {trace.outcome !== 'delivered' && words && hasDetail(words) && explain !== 'locked' && (
        <details className="why" open={explain === 'open'}>
          <summary>Why, and what to do</summary>
          <ExplainSections explanation={words} only={['what_happened', 'concept', 'how_to_fix']} />
          {trace.reason && (
            <a className="link" href={href.concept(trace.reason)}>
              Read the full page
            </a>
          )}
        </details>
      )}
      {trace.outcome !== 'delivered' && explain === 'locked' && hasDetail(words) && (
        <p className="result__estimate">The explanation opens once you finish this mission. Switch to Guided to read it now.</p>
      )}
    </section>
  );
}
