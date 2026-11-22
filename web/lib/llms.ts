import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { source } from '@/lib/source';

export function llmsIndex(): string {
  const pages = source.getPages();
  return [
    '# CP Lang',
    '',
    '> A competitive-programming language that compiles to one readable C++17 file. Beta (0.3).',
    '',
    'The full documentation as one file: /llms-full.txt',
    '',
    '## Docs',
    '',
    ...pages.map((p) => `- [${p.data.title}](${p.url}): ${p.data.description ?? ''}`),
    '',
  ].join('\n');
}

export function llmsFull(): string {
  return source
    .getPages()
    .map((p) => {
      const raw = readFileSync(join(process.cwd(), 'content/docs', p.path), 'utf8').replace(/^---[\s\S]*?---\n/, '');
      return `# ${p.data.title}\n\nURL: ${p.url}\n\n${raw.trim()}\n`;
    })
    .join('\n---\n\n');
}
