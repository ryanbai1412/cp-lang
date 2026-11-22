'use client';

import { useEffect, useState } from 'react';
import { decodeShare, Runner } from '@/components/runner';
import { tools } from '@/lib/tools';

const compilerVersion = tools.version.replace(/^cpl /, '');

const starter = `let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
print(a.sum())
`;

export function Playground() {
  const [initial, setInitial] = useState<{ src: string; in: string } | null>(null);
  const [notice, setNotice] = useState('');
  useEffect(() => {
    const fallback = { src: starter, in: '5\n3 1 4 1 5\n' };
    const id = /^\/p\/([0-9A-Za-z]{10})\/?$/.exec(location.pathname)?.[1];
    if (!id) return setInitial(decodeShare(location.hash) ?? fallback);
    fetch(`/api/p/${id}`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((s: { src: string; in: string; version: string }) => {
        if (s.version !== compilerVersion) setNotice(`Made with cpl ${s.version}; running on cpl ${compilerVersion}.`);
        setInitial({ src: s.src, in: s.in });
      })
      .catch(() => {
        setNotice('This share link could not be loaded.');
        setInitial(fallback);
      });
  }, []);
  return (
    <main className="container mx-auto max-w-5xl px-4 py-8">
      <h1 className="mb-4 text-2xl font-semibold tracking-tight">Playground</h1>
      {notice && <p className="mb-3 text-sm text-muted-foreground">{notice}</p>}
      {initial && (
        <Runner source={initial.src} stdin={initial.in} editorHeight="320px" playgroundLink={false} shareMenu />
      )}
    </main>
  );
}
