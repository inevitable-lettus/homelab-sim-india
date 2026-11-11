// The small subset of Markdown the explanations use: paragraphs, flat lists,
// numbered lists, links, **bold**, *emphasis* and `code`. Text is never
// injected as HTML.
import type { ReactNode } from 'react';

const INLINE = /(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)\s]+\))/g;

function inline(text: string): ReactNode[] {
  return text.split(INLINE).map((part, i) => {
    if (part.startsWith('**')) return <strong key={i}>{part.slice(2, -2)}</strong>;
    if (part.startsWith('`')) return <code key={i}>{part.slice(1, -1)}</code>;
    if (part.startsWith('*')) return <em key={i}>{part.slice(1, -1)}</em>;
    const link = /^\[([^\]]+)\]\(([^)\s]+)\)$/.exec(part);
    if (link?.[1] && link[2] && /^https?:\/\//.test(link[2])) {
      return (
        <a key={i} href={link[2]} target="_blank" rel="noreferrer noopener">
          {link[1]}
        </a>
      );
    }
    return part;
  });
}

export function Markdown({ text }: { text: string }) {
  const blocks = text.trim().split(/\n\s*\n/);
  return (
    <div className="prose">
      {blocks.map((block, i) => {
        const lines = block.split('\n');
        const bullets = lines.every((line) => /^\s*-\s+/.test(line) || /^\s{2,}\S/.test(line));
        const numbers = lines.every((line) => /^\s*\d+\.\s+/.test(line) || /^\s{2,}\S/.test(line));
        if (bullets || numbers) {
          const items: string[] = [];
          for (const line of lines) {
            const start = /^\s*(?:-|\d+\.)\s+(.*)$/.exec(line);
            if (start?.[1] !== undefined) items.push(start[1]);
            else if (items.length) items[items.length - 1] += ` ${line.trim()}`;
          }
          const List = numbers && !bullets ? 'ol' : 'ul';
          return (
            <List key={i}>
              {items.map((item, j) => (
                <li key={j}>{inline(item)}</li>
              ))}
            </List>
          );
        }
        return <p key={i}>{inline(lines.join(' '))}</p>;
      })}
    </div>
  );
}
