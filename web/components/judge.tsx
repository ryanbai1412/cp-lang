import type { ReactNode } from 'react';
import { Solve } from '@/components/exercise';
import { problem } from '@/lib/practice';

export function Judge({ id, children }: { id: string; children?: ReactNode }) {
  const p = problem(id);
  return (
    <Solve id={p.id} starter={p.starter} reference={p.reference} tests={p.tests}>
      {children}
    </Solve>
  );
}
