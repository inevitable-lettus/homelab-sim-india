// The core screen: brief and findings beside the board, playback and the step
// log under it. On a phone, one panel at a time.
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from 'react';
import { ArrowLeftIcon, GearIcon } from '../components/icons';
import { useMedia } from '../lib/media';
import { duration } from '../lib/motion';
import { TimelinePlayer } from '../lib/player';
import { progress, useProgress, useReducedMotion } from '../lib/progress';
import { href } from '../lib/route';
import { grade } from '../lib/sim';
import { buildTimeline } from '../lib/timeline';
import type { Bundle, Depth, Grade, Id, Mission, Trace, World } from '../lib/types';
import { setupKey } from '../lib/world';
import { Board } from '../mission/Board';
import { Inspector } from '../mission/Inspector';
import { PlayerBar } from '../mission/PlayerBar';
import { Result } from '../mission/Result';
import { editableNodes } from '../mission/Setup';
import { Brief, Debrief, Findings, goalRevealed } from '../mission/Side';
import { StepLog } from '../mission/StepLog';

type Tab = 'brief' | 'board' | 'findings';

function DepthSwitch({ depth }: { depth: Depth }) {
  return (
    <div className="depth" role="radiogroup" aria-label="Depth">
      {(['guided', 'challenge'] as const).map((value) => (
        <button key={value} type="button" role="radio" aria-checked={depth === value} className="seg seg--small" onClick={() => progress.update({ depth: value })} data-testid={`depth-${value}`}>
          {value === 'guided' ? 'Guided' : 'Challenge'}
        </button>
      ))}
    </div>
  );
}

/** A saved board from an older version may no longer be readable. Then start fresh. */
function startingWorld(mission: Mission, depth: Depth): World {
  const draft = progress.get().drafts[mission.id];
  if (!draft) return mission.world;
  try {
    grade(mission.id, depth, draft);
    return draft;
  } catch {
    progress.dropDraft(mission.id);
    return mission.world;
  }
}

const NO_RUNS: ReadonlySet<Id> = new Set();

function MissionPlay({ mission, bundle }: { mission: Mission; bundle: Bundle }) {
  const saved = useProgress();
  const depth = saved.depth;
  const reduced = useReducedMotion();
  const narrow = useMedia('(max-width: 767px)');

  const [world, setWorld] = useState(() => startingWorld(mission, depth));
  const [attemptId, setAttemptId] = useState<Id>(mission.world.attempts[0]?.id ?? '');
  const [selected, setSelected] = useState<Id | null>(null);
  const [tab, setTab] = useState<Tab>('brief');
  const [runs, setRuns] = useState<{ key: string; ids: Id[] }>({ key: '', ids: [] });

  const change = useCallback(
    (next: World) => {
      setWorld(next);
      progress.saveDraft(mission.id, next);
    },
    [mission.id],
  );

  const result: Grade = useMemo(() => grade(mission.id, depth, world), [mission.id, depth, world]);
  const trace: Trace | undefined = result.report.attempts.find((t) => t.attempt === attemptId);

  // Answering a question makes a new grade but the same trace. Key the
  // timeline on the trace's content so playback is not thrown away.
  const traceJson = JSON.stringify(trace ?? null);
  const timeline = useMemo(() => {
    const parsed = JSON.parse(traceJson) as Trace | null;
    return buildTimeline(parsed ?? { attempt: '', status: 'unknown', outcome: 'unknown', ended_at: null, reason: null, target: null, legs: [], summary: { hops: 0, one_way_ns: null, round_trip_ns: null, path_mtu: null, transfer: null } }, mission.world.links, duration);
  }, [traceJson, mission.world.links]);
  const player = useMemo(() => new TimelinePlayer(timeline), [timeline]);
  useEffect(() => () => player.dispose(), [player]);
  const playerState = useSyncExternalStore(player.subscribe, player.getState);

  const key = useMemo(() => setupKey(world), [world]);
  useEffect(
    () =>
      player.subscribe((state) => {
        if (!state.ended) return;
        setRuns((before) => {
          if (before.key !== key) return { key, ids: [attemptId] };
          return before.ids.includes(attemptId) ? before : { key, ids: [...before.ids, attemptId] };
        });
      }),
    [player, key, attemptId],
  );
  const ran = useMemo(() => (runs.key === key ? new Set(runs.ids) : NO_RUNS), [runs, key]);

  const goals = mission.goals.filter((goal) => !goal.depth || goal.depth === depth);
  const won = result.status === 'pass' && goals.every((goal) => goalRevealed(goal, ran));
  useEffect(() => {
    if (won) progress.finish(mission.id, depth);
  }, [won, mission.id, depth]);

  const finished = won || !!saved.done[mission.id]?.[depth];
  const explain = depth === 'guided' ? 'open' : finished ? 'closed' : 'locked';
  const editable = useMemo(() => editableNodes(mission), [mission]);
  const order = Object.values(bundle.missions).sort((a, b) => a.order - b.order);
  const next = order[order.findIndex((m) => m.id === mission.id) + 1];
  const showResult = !!trace && (ran.has(attemptId) || playerState.ended);

  const reset = () => {
    progress.dropDraft(mission.id);
    setWorld(mission.world);
    setSelected(null);
  };

  return (
    <div className="mission" data-tab={tab} data-won={won || undefined}>
      <header className="topbar">
        <a className="topbar__back" href={href.home}>
          <ArrowLeftIcon size={18} />
          <span>Track</span>
        </a>
        <h1 className="topbar__title">
          <span className="topbar__n">Mission {mission.order}</span> {mission.title}
        </h1>
        <DepthSwitch depth={depth} />
        <a className="icon-btn" href={href.settings} aria-label="Settings">
          <GearIcon />
        </a>
      </header>

      <div className="side">
      <div className="pane pane--brief" id="pane-brief" role={narrow ? 'tabpanel' : undefined} aria-label="Brief">
          <Brief mission={mission} depth={depth} world={world} grade={result} bundle={bundle} ran={ran} onChange={change} onReset={reset} />
        </div>
        <div className="pane pane--findings" id="pane-findings" role={narrow ? 'tabpanel' : undefined} aria-label="Findings">
          <Findings grade={result} bundle={bundle} world={world} ran={ran.size > 0} explain={explain} />
        </div>
      </div>

      <main className="pane pane--board" id="pane-board" role={narrow ? 'tabpanel' : undefined} aria-label="Board">
        <div className="stage">
          <Board
            world={world}
            facts={result.report.facts.nodes}
            positions={narrow ? mission.layout.narrow : mission.layout.wide}
            mode={narrow ? 'narrow' : 'wide'}
            player={player}
            editable={editable}
            selected={selected}
            onOpen={setSelected}
            covered={!narrow && selected ? 364 : 0}
            label={`Network board for ${mission.title}. The step log below describes the same thing in text.`}
          />
          {selected && (
            <Inspector nodeId={selected} mission={mission} world={world} facts={result.report.facts.nodes} bundle={bundle} onChange={change} onClose={() => setSelected(null)} />
          )}
        </div>
        <PlayerBar player={player} attempts={world.attempts} attemptId={attemptId} onAttempt={setAttemptId} reduced={reduced} />
        <div className="dock">
          {won && <Debrief mission={mission} grade={result} bundle={bundle} next={next} world={world} onReplay={() => player.reset()} />}
          {showResult && trace && <Result trace={trace} world={world} bundle={bundle} explain={explain} />}
          <StepLog player={player} world={world} bundle={bundle} />
        </div>
      </main>

      <nav className="tabs" role="tablist" aria-label="Panels">
        {(['brief', 'board', 'findings'] as const).map((value) => (
          <button key={value} type="button" role="tab" aria-selected={tab === value} aria-controls={`pane-${value}`} className="tabs__tab" onClick={() => setTab(value)} data-testid={`tab-${value}`}>
            {value === 'brief' ? 'Brief' : value === 'board' ? 'Board' : 'Findings'}
          </button>
        ))}
      </nav>
    </div>
  );
}

export function MissionScreen({ bundle, id }: { bundle: Bundle; id: Id }) {
  const mission = bundle.missions[id];
  if (!mission) {
    return (
      <div className="page">
        <h1>No such mission</h1>
        <p>
          <a className="link" href={href.home}>
            Back to the track
          </a>
        </p>
      </div>
    );
  }
  return <MissionPlay key={mission.id} mission={mission} bundle={bundle} />;
}
