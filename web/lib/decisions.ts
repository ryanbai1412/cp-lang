import { readFileSync } from 'node:fs';
import { join } from 'node:path';

// decisions.md, split for /decisions: one page per `## N.` entry, and within
// it one part per `- **N.M` sub-decision, each with its own anchor.
export type Part = { id?: string; text: string };
export type Decision = { slug: string; number: string; title: string; parts: Part[] };

const sub = /^- \*\*(\d+\.\d+) /;
let cache: Decision[] | undefined;

export function decisions(): Decision[] {
  if (cache) return cache;
  const text = readFileSync(join(process.cwd(), '..', 'decisions.md'), 'utf8');
  const slugs = new Set<string>();
  const subs = new Set<string>();
  cache = text
    .split(/^(?=## )/m)
    .slice(1)
    .map((section) => {
      const head = section.slice(3, section.indexOf('\n'));
      const number = /^(\d+(?:\.\d+)?)\. /.exec(head)?.[1] ?? '';
      let slug = number.replace('.', '-');
      for (let k = 2; slugs.has(slug); k++) slug = `${number.replace('.', '-')}-${'abcdefghij'[k - 2]}`;
      slugs.add(slug);
      const parts: Part[] = [{ text: '' }];
      for (const line of section.split('\n').slice(1)) {
        const m = sub.exec(line);
        if (m) parts.push({ id: subs.has(m[1]) ? undefined : `d-${m[1]}`, text: '' });
        if (m) subs.add(m[1]);
        parts[parts.length - 1].text += `${line}\n`;
      }
      return { slug, number, title: head.slice(number.length + 2), parts };
    });
  return cache;
}

// The page for a cited number: `128` is its own page, `14.2` an anchor on 14's.
export function decisionHrefs(): Map<string, string> {
  const hrefs = new Map<string, string>();
  for (const d of decisions()) {
    if (!hrefs.has(d.number)) hrefs.set(d.number, `/decisions/${d.slug}`);
    for (const p of d.parts) {
      const n = p.id?.slice(2);
      if (n && !hrefs.has(n)) hrefs.set(n, `/decisions/${d.slug}#${p.id}`);
    }
  }
  return hrefs;
}
