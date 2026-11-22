import type { Metadata } from 'next';
import { debugDemo, heroPrograms, libraryPrograms } from '@/lib/landing';
import { Playground } from './playground';

export const metadata: Metadata = {
  title: 'Playground',
  description: 'Write, run, trace and stress test CP Lang in your browser.',
};

export default async function Page() {
  const debug = debugDemo();
  const live = (await Promise.all([heroPrograms(), libraryPrograms()])).flat().filter((e) => e.status === 'runs');
  const examples = [
    ...live,
    { id: 'stress', title: 'Stress test a planted bug', source: debug.source, stdin: debug.stdin },
  ].map(({ id, title, source, stdin }) => ({ id, title, source, stdin }));
  return <Playground examples={examples} />;
}
