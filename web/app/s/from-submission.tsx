'use client';

import Link from 'next/link';
import { useMemo, useState } from 'react';
import { Runner } from '@/components/runner';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { extractSource } from '@/lib/extract';

const langs = [
  { href: '/learn/from/cpp', label: 'C++' },
  { href: '/learn/from/python', label: 'Python' },
  { href: '/learn/from/java', label: 'Java' },
  { href: '/learn/from/rust', label: 'Rust' },
];

export function FromSubmission() {
  const [cpp, setCpp] = useState('');
  const found = useMemo(() => (cpp.trim() ? extractSource(cpp) : null), [cpp]);
  const cppLines = cpp.replace(/\n+$/, '').split('\n').length;
  return (
    <main className="container mx-auto max-w-6xl px-4 py-12">
      <p className="text-sm text-muted-foreground">Beta · CP Lang 0.3</p>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">That C++ was written by CP Lang</h1>
      <p className="mt-2 max-w-3xl text-muted-foreground">
        CP Lang is a language for competitive programming that compiles to one readable C++17 file. The file quotes its
        own source at the top. Paste the whole submission to read it, change it and run it. Everything stays in your
        browser.
      </p>
      <div className="mt-8 grid gap-6 lg:grid-cols-2">
        <section className="space-y-2">
          <label htmlFor="cpp" className="text-sm font-medium">
            Submitted C++
          </label>
          <Textarea
            id="cpp"
            value={cpp}
            onChange={(e) => setCpp(e.target.value)}
            placeholder="// name.cp: C++17 written by cpl … from the CP Lang source below."
            className="h-[420px] font-mono text-xs"
            spellCheck={false}
          />
        </section>
        <section className="space-y-2">
          <p className="text-sm font-medium">
            CP Lang source
            {found && (
              <span className="ml-2 font-normal text-muted-foreground">
                {found.source.trimEnd().split('\n').length} lines, from {cppLines} lines of C++ · cpl {found.version}
              </span>
            )}
          </p>
          {found ? (
            <Runner key={found.source} source={found.source} editorHeight="260px" checkOnLoad />
          ) : (
            <Alert>
              <AlertDescription>
                {cpp.trim()
                  ? 'No CP Lang header found. The first lines of a generated file say “C++17 written by cpl … from the CP Lang source below.”'
                  : 'Paste a submission on the left.'}
              </AlertDescription>
            </Alert>
          )}
        </section>
      </div>
      <section className="mt-12 border-t pt-8">
        <h2 className="text-xl font-semibold tracking-tight">Try it on a problem</h2>
        <p className="mt-1 text-muted-foreground">Six short problems, side by side with the language you already know.</p>
        <div className="mt-4 flex flex-wrap gap-2">
          {langs.map((l, i) => (
            <Button key={l.href} variant={i === 0 ? 'default' : 'outline'} asChild>
              <Link href={l.href}>Coming from {l.label}</Link>
            </Button>
          ))}
        </div>
      </section>
    </main>
  );
}
