'use client';

import { ChevronDown } from 'lucide-react';
import { useEffect, useState } from 'react';
import { decodeShare, Runner } from '@/components/runner';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { tools } from '@/lib/tools';

const compilerVersion = tools.version.replace(/^cpl /, '');

export interface Example {
  id: string;
  title: string;
  source: string;
  stdin: string;
}

export function Playground({ examples }: { examples: Example[] }) {
  const [initial, setInitial] = useState<{ src: string; in: string } | null>(null);
  const [notice, setNotice] = useState('');
  useEffect(() => {
    const fallback = { src: examples[0].source, in: examples[0].stdin };
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
  }, [examples]);
  return (
    <main className="container mx-auto max-w-6xl px-4 py-6">
      <div className="mb-4 flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Playground</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            The real compiler, running in your browser. Run, step through a trace, or stress test against a brute force.
          </p>
        </div>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              Load an example <ChevronDown />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            {examples.map((e) => (
              <DropdownMenuItem
                key={e.id}
                onSelect={() => {
                  setNotice('');
                  setInitial({ src: e.source, in: e.stdin });
                }}
              >
                {e.title}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      {notice && <p className="mb-3 text-sm text-muted-foreground">{notice}</p>}
      {initial && (
        <Runner key={initial.src} source={initial.src} stdin={initial.in} editorHeight="max(320px, calc(100vh - 26rem))" playgroundLink={false} shareMenu checkOnLoad />
      )}
    </main>
  );
}
