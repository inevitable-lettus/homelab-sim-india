// Pass, fail and unknown always carry an icon and a word, never colour alone.
import type { Provenance, Status } from '../lib/types';
import { CheckIcon, CrossIcon, DotIcon, QuestionIcon } from './icons';

/** Design.md, "Findings and verdict language". */
export function verdictWord(status: Status, provenance: Provenance): string {
  if (status === 'unknown') return "We don't have data for this";
  if (status === 'fail') return 'Problem';
  switch (provenance) {
    case 'official':
      return 'Listed by the maker';
    case 'community':
      return 'Reported to work';
    case 'line_check':
      return 'Based on your line check';
    case 'teaching':
      return 'In this model';
  }
}

export function StatusMark({ status, word }: { status: Status | 'waiting'; word: string }) {
  const Icon = status === 'pass' ? CheckIcon : status === 'fail' ? CrossIcon : status === 'unknown' ? QuestionIcon : DotIcon;
  return (
    <span className="status" data-status={status}>
      <Icon size={16} />
      <span>{word}</span>
    </span>
  );
}
