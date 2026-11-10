//! Grading: did the learner's world meet the mission's goals? This is a pure
//! function of the mission, the world and the files the learner wrote.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::artifacts::{ArtifactResult, apply};
use crate::bundle::{AnswerCheck, Bundle, Depth, GoalCheck, Mission};
use crate::facts::Mode;
use crate::model::{Id, World};
use crate::reason::Reason;
use crate::trace::Outcome;
use crate::{Report, Status, evaluate};

/// Files read from the terminal or the editor, keyed by guest path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GoalResult {
    pub id: Id,
    pub status: Status,
    pub reason: Reason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub mission: Id,
    pub depth: Depth,
    pub status: Status,
    pub goals: Vec<GoalResult>,
    pub artifacts: Vec<ArtifactResult>,
    /// The evaluation the grade was based on, with the files applied.
    pub report: Report,
}

pub fn grade(mission: &Mission, depth: Depth, world: &World, artifacts: &Artifacts, bundle: &Bundle) -> Grade {
    let (world, artifact_results) = apply(mission, world, artifacts);
    let report = evaluate(&world, bundle);
    let goals: Vec<GoalResult> = mission
        .goals
        .iter()
        .filter(|goal| goal.depth.is_none_or(|d| d == depth))
        .map(|goal| {
            let (status, reason) = check(&goal.check, mission, &world, &report);
            GoalResult { id: goal.id.clone(), status, reason }
        })
        .collect();
    let statuses = goals.iter().map(|g| g.status).chain(artifact_results.iter().map(|a| a.status));
    let mut status = Status::Pass;
    for s in statuses {
        match s {
            Status::Fail => status = Status::Fail,
            Status::Unknown if status == Status::Pass => status = Status::Unknown,
            _ => {}
        }
    }
    Grade { mission: mission.id.clone(), depth, status, goals, artifacts: artifact_results, report }
}

fn check(goal: &GoalCheck, mission: &Mission, world: &World, report: &Report) -> (Status, Reason) {
    let unknown = (Status::Unknown, Reason::GoalUnknown);
    let trace = |id: &Id| report.attempts.iter().find(|t| t.attempt == *id);
    match goal {
        GoalCheck::Delivered { attempt, to, max_rtt_ms } => {
            let Some(trace) = trace(attempt) else {
                return unknown;
            };
            match trace.outcome {
                Outcome::Unknown => unknown,
                Outcome::Dropped => (Status::Fail, Reason::GoalNotMet),
                Outcome::Delivered => {
                    if to.is_some() && *to != trace.ended_at {
                        return (Status::Fail, Reason::GoalWrongNode);
                    }
                    match (max_rtt_ms, trace.summary.round_trip_ns) {
                        (None, _) => (Status::Pass, Reason::GoalMet),
                        (Some(_), None) => unknown,
                        (Some(max), Some(ns)) if ns > u64::from(*max) * 1_000_000 => {
                            (Status::Fail, Reason::GoalTooSlow)
                        }
                        (Some(_), Some(_)) => (Status::Pass, Reason::GoalMet),
                    }
                }
            }
        }
        GoalCheck::Reaches { attempt, node } => {
            let Some(trace) = trace(attempt) else {
                return unknown;
            };
            let reached = trace.request().is_some_and(|leg| leg.steps.iter().any(|step| step.at == *node));
            match (reached, trace.outcome) {
                (true, _) => (Status::Pass, Reason::GoalMet),
                (false, Outcome::Unknown) => unknown,
                (false, _) => (Status::Fail, Reason::GoalNotMet),
            }
        }
        GoalCheck::Answer { question } => {
            let Some(question) = mission.questions.iter().find(|q| q.id == *question) else {
                return unknown;
            };
            let Some(answer) = world.answers.iter().find(|a| a.question == question.id) else {
                return (Status::Fail, Reason::GoalUnanswered);
            };
            let expected = match &question.check {
                AnswerCheck::LineType { isp } => {
                    let line = report.facts.nodes.iter().find(|n| n.id == *isp).and_then(|n| n.line.as_ref());
                    match line.map(|line| line.ipv4) {
                        Some(Mode::Public) => Some("same".to_owned()),
                        Some(Mode::Cgnat) => Some("different".to_owned()),
                        _ => None,
                    }
                }
                AnswerCheck::BlockedAt { attempt } => trace(attempt).and_then(|trace| match trace.outcome {
                    Outcome::Delivered => Some("nowhere".to_owned()),
                    Outcome::Dropped => trace.ended_at.clone(),
                    Outcome::Unknown => None,
                }),
            };
            match expected {
                None => unknown,
                Some(expected) if expected == answer.value => (Status::Pass, Reason::GoalMet),
                Some(_) => (Status::Fail, Reason::GoalWrongAnswer),
            }
        }
        GoalCheck::RulePasses { rule } => {
            // A rule that said nothing has not passed.
            let mut matching = report.findings.iter().filter(|f| f.rule_id.id() == rule).peekable();
            if matching.peek().is_none() {
                return (Status::Fail, Reason::GoalNotMet);
            }
            let mut status = Status::Pass;
            for finding in matching {
                match finding.status {
                    Status::Fail => status = Status::Fail,
                    Status::Unknown if status == Status::Pass => status = Status::Unknown,
                    _ => {}
                }
            }
            match status {
                Status::Pass => (Status::Pass, Reason::GoalMet),
                Status::Fail => (Status::Fail, Reason::GoalNotMet),
                Status::Unknown => unknown,
            }
        }
    }
}
