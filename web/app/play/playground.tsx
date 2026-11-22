'use client';

import { useEffect, useState } from 'react';
import { decodeShare, Runner } from '@/components/runner';

const starter = `let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
print(a.sum())
`;

export function Playground() {
  const [initial, setInitial] = useState<{ src: string; in: string } | null>(null);
  useEffect(() => setInitial(decodeShare(location.hash) ?? { src: starter, in: '5\n3 1 4 1 5\n' }), []);
  return (
    <main className="container mx-auto max-w-5xl px-4 py-8">
      <h1 className="mb-4 text-2xl font-semibold tracking-tight">Playground</h1>
      {initial && <Runner source={initial.src} stdin={initial.in} editorHeight="320px" playgroundLink={false} />}
    </main>
  );
}
