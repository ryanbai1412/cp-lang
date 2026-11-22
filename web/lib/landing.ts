import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Status } from '@/components/status-badge';
import { highlight } from '@/lib/highlight';

const dir = join(process.cwd(), 'content/landing');

export interface Showcase {
  id: string;
  title: string;
  caption: string;
  status: Status;
  lines: number;
  html: string;
  href?: string;
}

const syntax = [
  {
    id: 'input',
    title: 'Typed input',
    caption:
      "The statement's format and limits are written once. They drive parsing, validation and the width of every value.",
  },
  {
    id: 'commands',
    title: 'Commands',
    caption: "A switch on 'I', 'Q' and 'E' with hand-written reads becomes an enum, and match checks every case is handled.",
  },
  {
    id: 'records',
    title: 'Records',
    caption: 'Named fields come straight from the input. An unreachable answer is None, printed as -1 in one place.',
  },
  {
    id: 'tables',
    title: 'Tables',
    caption: 'Build a table over an index space and ask a question about all of it, the way the editorial writes it.',
  },
];

const library = [
  {
    id: 'btoi20p3',
    title: 'Rollback DSU as a queue',
    caption:
      'BOI 2020 Joker, the whole solution. The editorial needs divide and conquer over a rollback DSU; here minimal finds the shortest windows with an odd cycle and undoing lets the DSU drop from the front.',
  },
  {
    id: 'ioi05p2',
    title: 'Lazy segment tree',
    caption:
      'IOI 2005 Mountain, the whole solution. You write the monoid and the action; SegTree gives range assign, binary search on the tree and coordinate compression through pieces.',
  },
  {
    id: 'cco21p4',
    title: 'Graph pipelines',
    caption:
      'CCO 2021 Travelling Merchant, the whole solution. Peel dead ends off the reversed graph, keep the rest, and settle answers from a heap.',
  },
  {
    id: 'egoi23p4',
    title: 'Algebra as values',
    caption:
      'EGOI 2023 Bikes vs Cars, the whole solution. One closure routine runs over two semirings, and Kruskal takes a key for the maximum forest.',
  },
];

async function load(group: string, items: { id: string; title: string; caption: string }[], link: boolean) {
  return Promise.all(
    items.map(async (t): Promise<Showcase> => {
      const source = readFileSync(join(dir, group, `${t.id}.spec.cp`), 'utf8').trimEnd();
      return {
        ...t,
        status: 'checks',
        lines: source.split('\n').length,
        html: await highlight(source, 'cp'),
        href: link && existsSync(join(process.cwd(), '../corpus/problems', t.id)) ? `/corpus/${t.id}` : undefined,
      };
    }),
  );
}

export interface Live {
  id: string;
  title: string;
  caption: string;
  source: string;
  stdin: string;
  cpp?: string;
  status: Status;
  html: string;
}

const hero = [
  {
    id: 'input',
    title: 'Typed input',
    caption:
      'The limits from the statement are the types. Change 90 to 900 in the input and run: the run stops and names the value.',
  },
  {
    id: 'prefix',
    title: 'Prefix sums',
    caption:
      'The array is 1-based because the statement is. Prefix takes the range the way the statement writes it, so there is no l - 1 to get wrong.',
  },
  {
    id: 'optional',
    title: 'No sentinels',
    caption: 'No answer yet is None, not -1 or INF. max= ignores it, and -1 appears once, where it is printed.',
  },
  {
    id: 'grid',
    title: 'Grids',
    caption: 'The type reads the grid. Rows and columns are different index spaces, so g[c, r] does not compile.',
  },
];

const lib = [
  {
    id: 'segtree',
    title: 'Segment tree',
    caption: 'Point updates and range minimum. You give SegTree the operation; fold takes the range as the statement writes it.',
  },
  {
    id: 'dijkstra',
    title: 'Dijkstra',
    caption: 'Shortest paths from vertex 1. Unreachable is None, so there is no INF to add to by mistake.',
  },
  { id: 'dsu', title: 'Union-find', caption: 'Dsu over the vertex domain: unite, same, size and the number of components.' },
  {
    id: 'mod',
    title: 'Mod and memo',
    caption: 'Counting paths in a DAG modulo 1e9 + 7. Mod carries the modulus in its type; @memo caches the recursion.',
  },
];

const gate = () =>
  JSON.parse(readFileSync(join(process.cwd(), '.gate/status.json'), 'utf8')) as Record<string, { status: Status }>;

function live(group: string, items: { id: string; title: string; caption: string }[]): Promise<Live[]> {
  const status = gate();
  return Promise.all(
    items.map(async (t) => {
      const base = join(dir, group, t.id);
      const file = existsSync(`${base}.cp`) ? `${t.id}.cp` : `${t.id}.spec.cp`;
      const source = readFileSync(join(dir, group, file), 'utf8');
      return {
        ...t,
        source,
        stdin: readFileSync(`${base}.in`, 'utf8'),
        cpp: existsSync(`${base}.cpp`) ? readFileSync(`${base}.cpp`, 'utf8').trimEnd() : undefined,
        status: status[`content/landing/${group}/${file}`].status,
        html: await highlight(source.trimEnd(), 'cp'),
      };
    }),
  );
}

export const heroPrograms = () => live('hero', hero);
export const libraryPrograms = () => live('lib', lib);

export const syntaxShowcases = () => load('syntax', syntax, false);
export const libraryShowcases = () => load('library', library, true);

export function debugDemo() {
  const read = (f: string) => readFileSync(join(dir, 'debug', f), 'utf8');
  const stress = JSON.parse(readFileSync(join(process.cwd(), 'content/generated/stress.json'), 'utf8')) as {
    command: string;
    output: string;
  };
  return { source: read('buy.cp'), stdin: read('buy.in'), stress };
}
