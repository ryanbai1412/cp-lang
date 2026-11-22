import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Status } from '@/components/status-badge';

const dir = join(process.cwd(), 'content/landing');

export interface Snippet {
  id: string;
  title: string;
  caption: string;
  status: Status;
  source: string;
  stdin: string;
}

const tabs: Omit<Snippet, 'source' | 'stdin'>[] = [
  {
    id: 'multitest',
    title: 'Multitest',
    status: 'runs',
    caption: 'The test count is read for you, and nothing leaks from one test to the next.',
  },
  {
    id: 'optionals',
    title: 'No sentinels',
    status: 'runs',
    caption: "`?Int` means 'maybe no answer'. No -1 or INF to forget.",
  },
  {
    id: 'domains',
    title: 'Index spaces',
    status: 'runs',
    caption: 'Positions of V cannot be mixed with other indices, and the input is validated against the statement.',
  },
];

export function landingSnippets(): Snippet[] {
  return tabs.map((t) => {
    const read = (ext: string) => {
      const p = join(dir, `${t.id}.${ext}`);
      return existsSync(p) ? readFileSync(p, 'utf8') : '';
    };
    return { ...t, source: read('cp'), stdin: read('in') };
  });
}
