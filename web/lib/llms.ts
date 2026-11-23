import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { decisions } from '@/lib/decisions';
import { source } from '@/lib/source';

type Node = ReturnType<typeof source.getPageTree>['children'][number];

const urls = (nodes: Node[]): string[] =>
  nodes.flatMap((n) => (n.type === 'page' ? [n.url] : n.type === 'folder' ? [...(n.index ? [n.index.url] : []), ...urls(n.children)] : []));

// Pages in sidebar order, so a model reads the guides from the tour on, as a visitor would.
function pages() {
  const order = urls(source.getPageTree().children);
  const rank = (url: string) => (order.includes(url) ? order.indexOf(url) : order.length);
  return source.getPages().sort((a, b) => rank(a.url) - rank(b.url));
}

export function llmsIndex(): string {
  return [
    '# CP Lang',
    '',
    '> A competitive-programming language that compiles to one readable C++17 file. Beta (0.3).',
    '',
    'The full documentation as one file: /llms-full.txt',
    '',
    '## Docs',
    '',
    ...pages().map((p) => `- [${p.data.title}](${p.url}): ${p.data.description ?? ''}`),
    '',
    '## Design decisions',
    '',
    'Why the language is the way it is: each entry gives the choice, the alternatives and how to reverse it.',
    '',
    ...decisionList(),
    '',
  ].join('\n');
}

export function llmsFull(): string {
  return pages()
    .map((p) => {
      const raw = readFileSync(join(process.cwd(), 'content/docs', p.path), 'utf8')
        .replace(/^---[\s\S]*?---\n/, '')
        // MDX cards read as a plain link list.
        .replace(/^[ \t]*<\/?Cards>\n/gm, '')
        .replace(/^[ \t]*<Card title="([^"]*)" href="([^"]*)" description="([^"]*)" \/>$/gm, '- [$1]($2): $3');
      return `# ${p.data.title}\n\nURL: ${p.url}\n\n${raw.trim()}\n`;
    })
    .concat(['# Design decisions\n\nURL: /decisions\n', ...decisionList(), ''].join('\n'))
    .join('\n---\n\n');
}

const decisionList = () => decisions().map((d) => `- [${d.number}. ${d.title}](/decisions/${d.slug})`);
