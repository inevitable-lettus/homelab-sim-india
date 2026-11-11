// The left-hand panel: the brief, the goals, what the learner can change,
// hints, findings and the debrief.
import { useState } from 'react';
import { ExplainSections, hasDetail } from '../components/Explain';
import { StatusMark, verdictWord } from '../components/Status';
import { ArrowRightIcon, BulbIcon } from '../components/icons';
import { href } from '../lib/route';
import type { Bundle, Depth, Finding, Grade, Id, Mission, World } from '../lib/types';
import { Controls, QuestionControl } from './Setup';

interface BriefProps {
  mission: Mission;
  depth: Depth;
  world: World;
  grade: Grade;
  bundle: Bundle;
  /** Attempts that have been run to the end with the current setup. */
  ran: ReadonlySet<Id>;
  onChange: (world: World) => void;
  onReset: () => void;
}

export function goalRevealed(goal: Mission['goals'][number], ran: ReadonlySet<Id>): boolean {
  const check = goal.check;
  if (check.kind === 'delivered' || check.kind === 'reaches') return ran.has(check.attempt);
  return ran.size > 0;
}

function Goals({ mission, depth, world, grade, bundle, ran, onChange }: Omit<BriefProps, 'onReset'>) {
  const goals = mission.goals.filter((goal) => !goal.depth || goal.depth === depth);
  return (
    <ol className="goals" data-testid="goals">
      {goals.map((goal) => {
        const result = grade.goals.find((g) => g.id === goal.id);
        const revealed = goalRevealed(goal, ran);
        const question = goal.check.kind === 'answer' ? mission.questions.find((q) => q.id === (goal.check as { question: Id }).question) : undefined;
        const status = !revealed || !result ? 'waiting' : result.status;
        const word =
          status === 'waiting'
            ? question && ran.size > 0
              ? 'Answer below'
              : 'Run to check'
            : (bundle.explanations[result?.reason ?? '']?.title ?? status);
        return (
          <li key={goal.id} className="goal" data-status={status} data-testid={`goal-${goal.id}`}>
            <div className="goal__head">
              <span className="goal__text">{goal.text}</span>
              <StatusMark status={status} word={word} />
            </div>
            {question && (
              <>
                <QuestionControl question={question} world={world} onChange={onChange} disabled={ran.size === 0} />
                {ran.size === 0 && <p className="goal__note">Run an attempt first, then answer.</p>}
                {revealed && result?.status === 'fail' && result.reason !== 'goal-unanswered' && (
                  <p className="goal__note">{bundle.explanations[result.reason]?.line}</p>
                )}
              </>
            )}
          </li>
        );
      })}
    </ol>
  );
}

function Hints({ hints, depth }: { hints: string[]; depth: Depth }) {
  const [shown, setShown] = useState(0);
  const limit = depth === 'challenge' ? Math.min(1, hints.length) : hints.length;
  if (limit === 0) return null;
  return (
    <section className="hints">
      <h3>Hints</h3>
      {shown > 0 && (
        <ol>
          {hints.slice(0, shown).map((hint) => (
            <li key={hint}>{hint}</li>
          ))}
        </ol>
      )}
      {shown < limit ? (
        <button type="button" className="btn" onClick={() => setShown(shown + 1)}>
          <BulbIcon size={18} />
          <span>{shown === 0 ? (depth === 'challenge' ? 'Show the one hint' : 'Show a hint') : `Show the next hint (${shown} of ${limit} shown)`}</span>
        </button>
      ) : (
        <p className="hints__done">{depth === 'challenge' ? 'That was the only hint at this depth.' : 'No more hints.'}</p>
      )}
    </section>
  );
}

export function Brief(props: BriefProps) {
  const { mission, depth, world, grade, onChange, onReset } = props;
  const text = mission[depth];
  return (
    <div className="brief">
      <header className="brief__head">
        <p className="brief__n">Mission {mission.order}</p>
        <h2>{mission.title}</h2>
        <p className="brief__concept">{mission.concept}</p>
      </header>
      <p className="brief__scenario">{mission.scenario}</p>
      <p className="brief__task">{text.brief}</p>

      <section>
        <h3>Goals</h3>
        <Goals {...props} />
      </section>

      {mission.controls.some((c) => c.kind !== 'question') && (
        <section>
          <h3>What you can change</h3>
          <Controls mission={mission} world={world} facts={grade.report.facts.nodes} onChange={onChange} />
          <button type="button" className="link-btn" onClick={onReset}>
            Put everything back as it was
          </button>
        </section>
      )}

      <Hints key={depth} hints={text.hints} depth={depth} />
    </div>
  );
}

function FindingItem({ finding, bundle, world, explain }: { finding: Finding; bundle: Bundle; world: World; explain: 'open' | 'closed' | 'locked' }) {
  const words = bundle.explanations[finding.explanation_id];
  const names = finding.involved.map((id) => world.nodes.find((n) => n.id === id)?.label).filter(Boolean);
  return (
    <li className="finding" data-status={finding.status} data-severity={finding.severity} data-testid="finding" data-rule={finding.rule_id}>
      <StatusMark status={finding.status} word={verdictWord(finding.status, finding.provenance)} />
      <h4>{words?.title ?? finding.explanation_id}</h4>
      <p>{words?.line}</p>
      {names.length > 0 && <p className="finding__where">{names.join(', ')}</p>}
      {hasDetail(words) && words && explain !== 'locked' && (
        <details className="why">
          <summary>Why, and what to do</summary>
          <ExplainSections explanation={words} only={['concept', 'how_to_fix', 'real_line']} />
          <a className="link" href={href.concept(finding.explanation_id)}>
            Read the full page
          </a>
        </details>
      )}
    </li>
  );
}

const SEVERITY = { error: 0, warning: 1, info: 2 } as const;

export function Findings({ grade, bundle, world, ran, explain }: { grade: Grade; bundle: Bundle; world: World; ran: boolean; explain: 'open' | 'closed' | 'locked' }) {
  const [all, setAll] = useState(false);
  // The core already ranks findings. This only keeps debrief-only notes out.
  const findings = grade.report.findings.filter((f) => !f.debrief_only).sort((a, b) => SEVERITY[a.severity] - SEVERITY[b.severity]);
  const shown = all ? findings : findings.slice(0, 3);
  return (
    <section className="findings" aria-label="Findings">
      <h3>Findings</h3>
      {!ran ? (
        <p className="empty">Run an attempt. What the simulator finds about this setup is listed here.</p>
      ) : findings.length === 0 ? (
        <p className="empty">Nothing to report about this setup.</p>
      ) : (
        <>
          <ul className="findings__list">
            {shown.map((finding, i) => (
              <FindingItem key={`${finding.rule_id}-${finding.involved.join()}-${i}`} finding={finding} bundle={bundle} world={world} explain={explain} />
            ))}
          </ul>
          {findings.length > 3 && (
            <button type="button" className="link-btn" onClick={() => setAll(!all)}>
              {all ? 'Show fewer' : `Show ${findings.length - 3} more`}
            </button>
          )}
        </>
      )}
    </section>
  );
}

export function Debrief({ mission, grade, bundle, next, world, onReplay }: { mission: Mission; grade: Grade; bundle: Bundle; next?: Mission; world: World; onReplay: () => void }) {
  const concept = bundle.explanations[mission.debrief.concept];
  const notes = grade.report.findings.filter((f) => f.debrief_only);
  return (
    <section className="debrief" data-testid="debrief" aria-labelledby="debrief-title">
      <StatusMark status="pass" word="Mission complete" />
      <h2 id="debrief-title">{mission.title}</h2>
      <h3>What you did</h3>
      <p>{mission.debrief.did}</p>
      {concept && (
        <>
          <h3>The idea: {concept.title}</h3>
          {concept.concept ? <ExplainSections explanation={concept} only={['concept']} /> : <p>{concept.line}</p>}
          <a className="link" href={href.concept(mission.debrief.concept)}>
            Read the full page
          </a>
        </>
      )}
      <h3>On a real line</h3>
      <p>{mission.debrief.real_line}</p>
      {notes.length > 0 && (
        <>
          <h3>Worth knowing</h3>
          <ul className="findings__list">
            {notes.map((finding) => (
              <FindingItem key={finding.rule_id} finding={finding} bundle={bundle} world={world} explain="closed" />
            ))}
          </ul>
        </>
      )}
      <div className="debrief__actions">
        {next ? (
          <a className="btn btn--primary" href={href.mission(next.id)} data-testid="next-mission">
            <span>Next: {next.title}</span>
            <ArrowRightIcon size={18} />
          </a>
        ) : (
          <a className="btn btn--primary" href={href.home}>
            <span>Back to the track</span>
            <ArrowRightIcon size={18} />
          </a>
        )}
        <button type="button" className="btn" onClick={onReplay}>
          Play it again
        </button>
      </div>
    </section>
  );
}
