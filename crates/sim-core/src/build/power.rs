//! Will it last: running cost, power cuts, and what long hours do to a laptop.
//! See context-docs/Sandbox.md sections 7.4 and 8.

use serde::Serialize;

use super::{Cx, MachineBudget};
use crate::model::Id;
use crate::reason::Reason;

/// A number the result depends on that we chose, not measured. Shown next to
/// the figures it affects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assumption {
    pub reason: Reason,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RateUsed {
    /// The tariff the slabs came from, or `None` for the learner's own rate.
    pub tariff: Option<Id>,
    /// Paise per kWh on the first unit the lab adds.
    pub from_paise_per_kwh: u32,
    /// Paise per kWh on the last unit the lab adds.
    pub to_paise_per_kwh: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PowerSummary {
    pub lab_avg_w: Option<u32>,
    pub lab_kwh_year: Option<u32>,
    pub yearly_paise: Option<u64>,
    pub rate_used: Option<RateUsed>,
    pub assumptions: Vec<Assumption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpsSummary {
    pub node: Id,
    pub fed: Vec<Id>,
    pub fed_avg_w: Option<u32>,
    pub runtime_min: Option<u32>,
    pub overloaded: Option<bool>,
}

pub(super) fn rules(_cx: &mut Cx, _machines: &mut [MachineBudget]) {}
