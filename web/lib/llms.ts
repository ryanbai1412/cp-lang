import { readFileSync } from 'node:fs';
import { join } from 'node:path';
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
  ].join('\n');
}

export function llmsFull(): string {
  return pages()
    .map((p) => {
      const raw = readFileSync(join(process.cwd(), 'content/docs', p.path), 'utf8').replace(/^---[\s\S]*?---\n/, '');
      return `# ${p.data.title}\n\nURL: ${p.url}\n\n${raw.trim()}\n`;
    })
    .join('\n---\n\n');
}
