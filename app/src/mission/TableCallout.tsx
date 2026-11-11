// What a NAT table or a list of port forwards did with the packet at one step.
import type { Id, Step, WorldNode } from '../lib/types';

interface Props {
  step: Step;
  nodeLabel: (id: Id) => string;
  nodeKind: (id: Id) => WorldNode['kind'] | undefined;
}

export function tableName(kind: WorldNode['kind'] | undefined): string {
  return kind === 'isp' ? 'Carrier NAT table' : kind === 'router' ? "Router's NAT table" : 'Table';
}

/** One sentence for a table event, used by the callout and by the step log. */
export function tableSummary(step: Step): { head: string; tone: 'new' | 'hit' | 'miss' } | null {
  const event = step.table;
  if (!event) return null;
  switch (event.kind) {
    case 'mapping_created':
      return { head: 'New row added', tone: 'new' };
    case 'mapping_hit':
      return { head: 'Matches an existing row', tone: 'hit' };
    case 'forward_hit':
      return { head: 'Matches a port forward', tone: 'hit' };
    case 'expired':
      return { head: 'The row has expired', tone: 'miss' };
    case 'no_match':
      return { head: 'No row and no forward matches', tone: 'miss' };
  }
}

export function TableCallout({ step, nodeLabel, nodeKind }: Props) {
  const summary = tableSummary(step);
  const event = step.table;
  if (!summary || !event) return null;
  const row = event.row;
  return (
    <div className="callout" data-tone={summary.tone}>
      <span className="callout__name">
        {tableName(nodeKind(step.at))} at {nodeLabel(step.at)}
      </span>
      <span className="callout__head">{summary.head}</span>
      {row ? (
        <span className="callout__row">
          <span>{row.inside}</span>
          <span className="callout__swap" aria-hidden="true">
            ⇄
          </span>
          <span>{row.outside}</span>
        </span>
      ) : (
        <span className="callout__row callout__row--empty">nothing for {step.packet.dst}</span>
      )}
      {row && event.kind !== 'forward_hit' && (
        <span className="callout__note">
          {row.timeout_s === null ? 'idle timeout unknown' : `forgotten after ${row.timeout_s} s idle`}
          {event.kind !== 'mapping_created' && `, last used ${row.idle_s} s ago`}
        </span>
      )}
    </div>
  );
}
