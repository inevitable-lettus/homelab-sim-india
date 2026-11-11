// Landing and track map in one: the promise, one real trace of a blocked
// connection, and the missions in order.
import { useEffect, useMemo, useSyncExternalStore } from 'react';
import { ArrowRightIcon, CheckIcon, LockIcon, RestartIcon } from '../components/icons';
import { duration } from '../lib/motion';
import { TimelinePlayer } from '../lib/player';
import { useProgress, useReducedMotion } from '../lib/progress';
import { href } from '../lib/route';
import { grade } from '../lib/sim';
import { sourceLink } from '../lib/source';
import { buildTimeline } from '../lib/timeline';
import type { Bundle, Mission } from '../lib/types';
import { Board } from '../mission/Board';

/** The blocked port forward from Mission 3, traced by the core and played once. */
function BlockedDemo({ mission, bundle }: { mission: Mission; bundle: Bundle }) {
  const reduced = useReducedMotion();
  const result = useMemo(() => grade(mission.id, 'guided', mission.world), [mission]);
  const trace = result.report.attempts[0];
  const player = useMemo(() => (trace ? new TimelinePlayer(buildTimeline(trace, mission.world.links, duration)) : null), [trace, mission]);
  const state = useSyncExternalStore(player?.subscribe ?? (() => () => {}), () => player?.getState() ?? null);

  useEffect(() => {
    if (!player) return;
    // Reduced motion gets the end state as a still picture.
    const start = setTimeout(() => (reduced ? player.toEnd() : player.play()), 500);
    return () => {
      clearTimeout(start);
      player.dispose();
    };
  }, [player, reduced]);

  if (!player || !trace) return null;
  const step = state && state.step >= 0 ? player.getTimeline().steps[state.step]?.step : undefined;
  const words = step ? bundle.explanations[step.reason] : undefined;
  const where = step ? mission.world.nodes.find((n) => n.id === step.at)?.label : undefined;
  return (
    <figure className="demo">
      <div className="demo__board">
        <Board world={mission.world} facts={result.report.facts.nodes} positions={mission.layout.narrow} mode="narrow" player={player} passive label="A phone on mobile data tries to open a server at home. The packet is stopped at the home ISP." />
      </div>
      <figcaption className="demo__caption">
        <p className="demo__step" aria-live="polite">
          {step ? (
            <>
              <strong>{where}.</strong> {words?.line}
            </>
          ) : (
            'A phone on mobile data tries to open a media server at home.'
          )}
        </p>
        <button type="button" className="btn" onClick={() => (reduced ? player.toEnd() : (player.reset(), player.play()))} disabled={state?.playing}>
          <RestartIcon size={18} />
          <span>Replay</span>
        </button>
      </figcaption>
    </figure>
  );
}

export function Home({ bundle }: { bundle: Bundle }) {
  const saved = useProgress();
  const track = bundle.tracks[0];
  const missions = (track?.missions ?? []).map((id) => bundle.missions[id]).filter((m): m is Mission => !!m);
  const isDone = (m: Mission) => !!saved.done[m.id]?.guided || !!saved.done[m.id]?.challenge;
  const next = missions.find((m) => !isDone(m)) ?? missions[0];
  const started = missions.some(isDone);
  const demo = bundle.missions['the-wall'];

  return (
    <div className="home">
      <header className="sitebar">
        <a className="sitebar__name" href={href.home}>
          Homelab Sim <span>India</span>
        </a>
        <nav aria-label="Site">
          <a href={href.settings}>Settings</a>
          <a href={href.credits}>Source and credits</a>
        </nav>
      </header>

      <main>
        <section className="hero">
          <div className="hero__text">
            <h1>You forwarded the port. It still doesn't work. Here is why.</h1>
            <p className="hero__lede">
              On many Indian broadband lines there is a second gate between you and the internet, and it is not yours to open. Play it out on a simulated network, watch the exact hop that stops your packet, and learn what works instead.
            </p>
            {next && (
              <a className="btn btn--primary btn--large" href={href.mission(next.id)} data-testid="start">
                <span>{started ? `Continue: ${next.title}` : `Start Mission 1: ${next.title}`}</span>
                <ArrowRightIcon size={18} />
              </a>
            )}
            <ul className="hero__facts">
              <li>Free and open source</li>
              <li>No signup</li>
              <li>Nothing you do leaves this browser</li>
            </ul>
          </div>
          {demo && <BlockedDemo mission={demo} bundle={bundle} />}
        </section>

        {track && (
          <section className="track" aria-labelledby="track-title">
            <div className="track__head">
              <h2 id="track-title">{track.title}</h2>
              <p>{track.summary}</p>
            </div>
            <ol className="track__list">
              {missions.map((mission) => {
                const done = saved.done[mission.id];
                return (
                  <li key={mission.id}>
                    <a className="track__item" href={href.mission(mission.id)} data-testid={`mission-${mission.id}`}>
                      <span className="track__n">{mission.order}</span>
                      <span className="track__what">
                        <span className="track__title">{mission.title}</span>
                        <span className="track__concept">{mission.concept}</span>
                      </span>
                      <span className="track__state" data-done={done ? '' : undefined}>
                        {done ? (
                          <>
                            <CheckIcon size={16} />
                            {done.challenge ? 'Done, both depths' : 'Done'}
                          </>
                        ) : mission.id === next?.id ? (
                          'Up next'
                        ) : (
                          'Not started'
                        )}
                      </span>
                    </a>
                  </li>
                );
              })}
              {track.planned.map((planned, i) => (
                <li key={planned.title}>
                  <div className="track__item track__item--planned">
                    <span className="track__n">{missions.length + i + 1}</span>
                    <span className="track__what">
                      <span className="track__title">{planned.title}</span>
                      <span className="track__concept">{planned.concept}</span>
                    </span>
                    <span className="track__state">
                      <LockIcon size={16} />
                      Planned
                    </span>
                  </div>
                </li>
              ))}
            </ol>
          </section>
        )}
      </main>

      <footer className="sitefoot">
        <p>The networks here are teaching models with made-up addresses. They say nothing about any real provider.</p>
        <p>
          Code is AGPL-3.0, lessons are CC BY-SA 4.0. {sourceLink ? <a href={sourceLink.href}>{sourceLink.label}</a> : <a href={href.credits}>Source and credits</a>}
        </p>
      </footer>
    </div>
  );
}
