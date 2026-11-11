// One explanation, in the six sections from Design.md. Written once in
// content/explanations and reused wherever its reason comes up.
import type { Explanation } from '../lib/types';
import { Markdown } from './Markdown';

const SECTIONS: [keyof Explanation, string][] = [
  ['what_happened', 'What happened'],
  ['concept', 'The idea'],
  ['why_it_matters', 'Why it matters'],
  ['how_to_fix', 'How to fix it'],
  ['real_line', 'On a real line'],
  ['go_deeper', 'Go deeper'],
];

export const hasDetail = (explanation: Explanation | undefined) =>
  !!explanation && SECTIONS.some(([key]) => explanation[key]);

export function ExplainSections({ explanation, only }: { explanation: Explanation; only?: (keyof Explanation)[] }) {
  return (
    <div className="explain">
      {SECTIONS.filter(([key]) => explanation[key] && (!only || only.includes(key))).map(([key, heading]) => (
        <section key={key}>
          <h4>{heading}</h4>
          <Markdown text={explanation[key] ?? ''} />
        </section>
      ))}
    </div>
  );
}
