// What the core said about the build. Every status, number and reason here
// comes from the report; this file only lays it out.
import { ExplainSections, hasDetail } from '../components/Explain';
import { StatusMark, verdictWord } from '../components/Status';
import { goalText, rupees } from '../lib/build';
import type { Catalog } from '../lib/packs';
import { sourceUrl } from '../lib/source';
import type { Budget, Bundle, Finding, Id, Report, World } from '../lib/types';
import { STATUS_WORD } from './Board';
import { Section } from './fields';

const UNITS: Record<string, string> = {
  'assumed-busy-share': 'parts in 1000 of the time',
  'assumed-base-use': 'kWh a month',
  'assumed-ups-usable': 'parts in 1000 of the battery',
  'assumed-laptop-usable': 'parts in 1000 of the battery',
};

function FindingRow({ finding, bundle, label }: { finding: Finding; bundle: Bundle; label: (id: Id) => string }) {
  const explanation = bundle.explanations[finding.explanation_id];
  return (
    <li className="bfinding" data-status={finding.status} data-testid={`finding-${finding.rule_id}`}>
      <StatusMark status={finding.status} word={finding.severity === 'warning' && finding.status === 'fail' ? 'Worth knowing' : verdictWord(finding.status, finding.provenance)} />
      <p className="bfinding__title">
        {explanation?.title ?? finding.rule_id}
        {finding.involved.length > 0 && <span className="bfinding__at"> · {finding.involved.map(label).join(', ')}</span>}
      </p>
      {explanation?.line && <p className="bfinding__line">{explanation.line}</p>}
      {explanation && hasDetail(explanation) && (
        <details>
          <summary>Why, and what to do</summary>
          <ExplainSections explanation={explanation} />
        </details>
      )}
    </li>
  );
}

function BudgetRow({ name, budget, unit }: { name: string; budget: Budget; unit: string }) {
  if (budget.total === null && budget.used === null && !budget.terms?.length) return null;
  const known = budget.used !== null && budget.total !== null;
  return (
    <div className="budget" data-over={known && budget.used! > budget.total!}>
      <p>
        <strong>{name}</strong>{' '}
        {known ? `${budget.used} of ${budget.total} ${unit}` : `${budget.used ?? "can't tell"} of ${budget.total ?? "can't tell"} ${unit}`}
        {budget.recommended !== undefined && budget.recommended !== budget.used && ` (${budget.recommended} at recommended sizes)`}
      </p>
      {budget.terms && budget.terms.length > 0 && (
        <ul className="budget__terms">
          {budget.terms.map((term, index) => (
            <li key={index}>
              {term.label}: {term.value ?? "can't tell"}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function helpLink(pack: Id, field: string): string | null {
  if (!sourceUrl) return null;
  // Only the pack and the field go in the link. Nothing about the build or the learner.
  const query = new URLSearchParams({ template: 'data.yml', title: `Data: ${pack} ${field}`, pack, field });
  return `${sourceUrl}/issues/new?${query.toString()}`;
}

export function ReportPanel({ report, world, catalog, bundle }: { report: Report; world: World; catalog: Catalog; bundle: Bundle }) {
  const build = report.build;
  const label = (id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id;
  if (!build) {
    return (
      <div className="breport" data-testid="report">
        <p className="bsec__note">Add a machine and a goal, and the report appears here.</p>
      </div>
    );
  }
  const counts = (finding: Finding) => finding.status !== 'pass' && finding.severity !== 'info';
  const fixFirst = report.findings.filter((f) => f.status === 'fail' && f.severity === 'error' && !f.debrief_only).slice(0, 3);
  const risks = report.findings.filter((f) => (f.debrief_only && f.rule_id !== 'learner-value-used') || (f.status === 'fail' && f.severity === 'warning'));
  const power = build.power;
  const tariff = power.rate_used?.tariff ? catalog.tariffs?.[power.rate_used.tariff] : undefined;
  const packLabel = (id: Id) => catalog.devices[id]?.label ?? catalog.os?.[id]?.label ?? catalog.services?.[id]?.label ?? catalog.runtimes?.[id]?.label ?? label(id);

  return (
    <div className="breport" data-testid="report">
      <Section title="Will it work?">
        {build.intents.length === 0 && <p className="bsec__note">No goals yet. Add one under Goals.</p>}
        <ul className="verdicts">
          {build.intents.map((result) => {
            const intent = world.intents?.find((i) => i.id === result.id);
            const under = result.findings.map((index) => report.findings[index]).filter((f): f is Finding => !!f);
            const first = under.find((f) => f.status === 'fail' && f.severity === 'error') ?? under.find(counts);
            const reason = first && bundle.explanations[first.explanation_id];
            return (
              <li key={result.id} data-testid={`verdict-${result.id}`} data-status={result.status}>
                <StatusMark status={result.status} word={STATUS_WORD[result.status]} />
                <p className="verdicts__goal">{intent ? goalText(intent, world, catalog) : result.id}</p>
                {reason && result.status !== 'pass' && <p className="bfinding__line">{reason.line || reason.title}</p>}
                {under.length > 0 && (
                  <details>
                    <summary>What was checked ({under.length})</summary>
                    <ul className="bfindings">
                      {under.map((finding, index) => (
                        <FindingRow key={index} finding={finding} bundle={bundle} label={label} />
                      ))}
                    </ul>
                  </details>
                )}
              </li>
            );
          })}
        </ul>
      </Section>

      {fixFirst.length > 0 && (
        <Section title="What to fix first">
          <ul className="bfindings">
            {fixFirst.map((finding, index) => (
              <FindingRow key={index} finding={finding} bundle={bundle} label={label} />
            ))}
          </ul>
        </Section>
      )}

      {build.machines.length > 0 && (
        <Section title="Fit">
          {build.machines.map((machine) => (
            <div key={machine.node} className="machine" data-testid={`fit-${machine.node}`}>
              <h4>{label(machine.node)}</h4>
              <BudgetRow name="Memory" budget={machine.ram} unit="MB" />
              <BudgetRow name="Drive bays" budget={machine.bays} unit="used" />
              <BudgetRow name="USB ports" budget={machine.usb} unit="used" />
              <BudgetRow name="Ethernet ports" budget={machine.ports} unit="used" />
            </div>
          ))}
        </Section>
      )}

      <Section title="Power and cost">
        <p className="bsec__note">Estimates, from the watts in each part's data and the assumptions listed below.</p>
        <dl className="bfacts" data-testid="power">
          {build.machines.map((machine) => (
            <div key={machine.node}>
              <dt>{label(machine.node)}</dt>
              <dd>
                {machine.avg_w === null ? "can't tell" : `about ${machine.avg_w} W on average`}
                {machine.battery_min !== undefined && `, about ${machine.battery_min} minutes on its own battery`}
              </dd>
            </div>
          ))}
          <div>
            <dt>The whole lab</dt>
            <dd>{power.lab_avg_w === null ? "can't tell" : `about ${power.lab_avg_w} W, ${power.lab_kwh_year ?? '?'} kWh a year`}</dd>
          </div>
          <div>
            <dt>Electricity a year</dt>
            <dd data-testid="yearly">{power.yearly_paise === null ? "can't tell yet: choose a tariff under Goals" : `about ${rupees(power.yearly_paise)}`}</dd>
          </div>
          {power.rate_used && (
            <div>
              <dt>Rate used</dt>
              <dd>
                {power.rate_used.from_paise_per_kwh === power.rate_used.to_paise_per_kwh
                  ? `${power.rate_used.from_paise_per_kwh} paise a unit`
                  : `${power.rate_used.from_paise_per_kwh} to ${power.rate_used.to_paise_per_kwh} paise a unit`}
                {tariff ? `, from ${tariff.discom}${tariff.teaching ? ' (a teaching tariff, not a real one)' : `, ${tariff.category}, retrieved ${tariff.retrieved_at}`}` : ', the rate you entered'}
              </dd>
            </div>
          )}
          {build.ups.map((ups) => (
            <div key={ups.node}>
              <dt>{label(ups.node)}</dt>
              <dd data-testid={`ups-${ups.node}`}>
                {ups.fed.length === 0
                  ? 'feeds nothing yet'
                  : `feeds ${ups.fed.map(label).join(', ')}: ${ups.runtime_min === null ? "can't tell how long" : `about ${ups.runtime_min} minutes`}${ups.overloaded ? ', and it is overloaded' : ''}`}
              </dd>
            </div>
          ))}
        </dl>
        {power.rate_used?.tariff && <p className="bsec__note">Your lab is billed on top of what your home already uses, so it pays your top slab, not the cheapest one.</p>}
        {power.assumptions.length > 0 && (
          <>
            <h4>Assumed, not measured</h4>
            <ul className="assumed">
              {power.assumptions.map((assumption) => (
                <li key={assumption.reason}>
                  <strong>{bundle.explanations[assumption.reason]?.title ?? assumption.reason}</strong>
                  {UNITS[assumption.reason] && `: ${assumption.value} ${UNITS[assumption.reason]}`}. {bundle.explanations[assumption.reason]?.line}
                </li>
              ))}
            </ul>
          </>
        )}
      </Section>

      <Section title="Parts list">
        <p className="bsec__note">Prices you entered. We don't ship prices.</p>
        <ul className="rows">
          {build.parts.map((part, index) => (
            <li key={index}>
              <span>{part.label}</span>
              <span>{part.price_paise === null ? 'no price' : `${rupees(part.price_paise)}${part.owned ? ', owned' : ''}`}</span>
            </li>
          ))}
        </ul>
        {build.price_total_paise !== null && (
          <p data-testid="total">
            <strong>To buy: {rupees(build.price_total_paise)}</strong>
          </p>
        )}
      </Section>

      {risks.length > 0 && (
        <Section title="Risks">
          <ul className="bfindings">
            {risks.map((finding, index) => (
              <FindingRow key={index} finding={finding} bundle={bundle} label={label} />
            ))}
          </ul>
        </Section>
      )}

      {build.unknowns.length > 0 && (
        <Section title="What we don't know">
          <ul className="unknowns">
            {build.unknowns.map((item) => {
              const link = helpLink(item.pack, item.field);
              return (
                <li key={`${item.pack}/${item.field}`} data-testid="unknown">
                  We have no data for <code>{item.field}</code> of {packLabel(item.pack)}. It affects: {item.used_by.map((reason) => bundle.explanations[reason]?.title ?? reason).join(', ')}.{' '}
                  {link && (
                    <a href={link} target="_blank" rel="noreferrer" data-testid="help-add">
                      Help add this
                    </a>
                  )}
                </li>
              );
            })}
          </ul>
        </Section>
      )}
    </div>
  );
}
