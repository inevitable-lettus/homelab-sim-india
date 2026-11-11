// Run, pause, step and scrub. The learner can always interrupt.
import { useEffect, useRef, useSyncExternalStore } from 'react';
import { PauseIcon, PlayIcon, RestartIcon, SkipIcon, StepBackIcon, StepForwardIcon } from '../components/icons';
import { playbackSpeeds } from '../lib/motion';
import type { TimelinePlayer } from '../lib/player';
import type { Attempt, Id } from '../lib/types';

interface Props {
  player: TimelinePlayer;
  attempts: Attempt[];
  attemptId: Id;
  onAttempt: (id: Id) => void;
  reduced: boolean;
  onRun?: () => void;
}

export function PlayerBar({ player, attempts, attemptId, onAttempt, reduced, onRun }: Props) {
  const state = useSyncExternalStore(player.subscribe, player.getState);
  const timeline = player.getTimeline();
  const count = timeline.steps.length;
  const fill = useRef<HTMLSpanElement>(null);
  const range = useRef<HTMLInputElement>(null);

  // The progress line follows the clock without re-rendering.
  useEffect(
    () =>
      player.onFrame((time) => {
        const share = timeline.total > 0 ? time / timeline.total : 0;
        if (fill.current) fill.current.style.transform = `scaleX(${share})`;
        if (range.current && document.activeElement !== range.current) range.current.value = String(Math.round(share * 1000));
      }),
    [player, timeline],
  );

  const started = state.step >= 0 || state.playing;
  const run = () => {
    onRun?.();
    if (reduced) {
      if (state.ended || !started) player.showStep(0);
      else player.stepForward();
    } else {
      player.toggle();
    }
  };
  const primary = reduced
    ? !started || state.ended
      ? 'Run'
      : 'Next step'
    : state.playing
      ? 'Pause'
      : state.ended
        ? 'Run again'
        : started
          ? 'Resume'
          : 'Run';

  return (
    <div className="player" role="group" aria-label="Playback">
      {attempts.length > 1 && (
        <div className="player__attempts" role="radiogroup" aria-label="What to try">
          {attempts.map((attempt) => (
            <button
              key={attempt.id}
              type="button"
              role="radio"
              aria-checked={attempt.id === attemptId}
              className="seg"
              onClick={() => onAttempt(attempt.id)}
            >
              {attempt.label}
            </button>
          ))}
        </div>
      )}
      <div className="player__row">
        <button type="button" className="btn btn--primary player__run" onClick={run} data-testid="run">
          {primary === 'Pause' ? <PauseIcon size={18} /> : <PlayIcon size={18} />}
          <span>{primary}</span>
        </button>
        <div className="player__steps">
          <button type="button" className="icon-btn" onClick={() => player.reset()} disabled={!started} aria-label="Back to the start">
            <RestartIcon />
          </button>
          <button type="button" className="icon-btn" onClick={() => player.stepBack()} disabled={state.step < 0} aria-label="Previous step">
            <StepBackIcon />
          </button>
          <button
            type="button"
            className="icon-btn"
            onClick={() => {
              onRun?.();
              player.stepForward();
            }}
            disabled={count === 0 ? state.ended : state.step >= count - 1 && state.ended}
            aria-label="Next step"
            data-testid="step-forward"
          >
            <StepForwardIcon />
          </button>
          <button
            type="button"
            className="icon-btn"
            onClick={() => {
              onRun?.();
              player.toEnd();
            }}
            disabled={state.ended}
            aria-label="Skip to the result"
            data-testid="skip"
          >
            <SkipIcon />
          </button>
        </div>
        <span className="player__count" aria-live="off">
          {count === 0 ? 'No packet sent' : `Step ${Math.max(0, state.step + 1)} of ${count}`}
        </span>
        {!reduced && (
          <div className="player__speed" role="radiogroup" aria-label="Speed">
            {playbackSpeeds.map((speed) => (
              <button
                key={speed}
                type="button"
                role="radio"
                aria-checked={state.speed === speed}
                className="seg seg--small"
                onClick={() => player.setSpeed(speed)}
              >
                {speed}x
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="player__track">
        <span ref={fill} className="player__fill" />
        <input
          ref={range}
          type="range"
          min={0}
          max={1000}
          defaultValue={0}
          aria-label="Scrub through the trace"
          disabled={count === 0}
          onChange={(event) => {
            onRun?.();
            player.seek((Number(event.target.value) / 1000) * timeline.total);
          }}
        />
      </div>
    </div>
  );
}
