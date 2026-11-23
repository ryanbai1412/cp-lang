'use client';

import { CheckCircle2, Loader2, Play, XCircle } from 'lucide-react';
import { type ReactNode, useEffect, useState } from 'react';
import { CodeEditor } from '@/components/editor/code-editor';
import { formatDiagnostics } from '@/components/runner';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { cpl } from '@/lib/cpl';
import type { Test } from '@/lib/practice';
import { cn } from '@/lib/utils';

type Verdict = { test: number; ok: boolean; got: string; expected: string; input: string; error?: string };

const norm = (s: string) => s.replace(/\s+$/g, '').split('\n').map((l) => l.trimEnd()).join('\n');

async function judge(source: string, tests: Test[]): Promise<Verdict[]> {
  const out: Verdict[] = [];
  for (const [i, t] of tests.entries()) {
    const r = await cpl.run(source, t.input);
    const error = r.ok ? undefined : formatDiagnostics(r.diagnostics, source) || r.stderr;
    out.push({ test: i + 1, ok: r.ok && norm(r.stdout) === norm(t.expected), got: r.stdout, expected: t.expected, input: t.input, error });
    if (!r.ok && r.diagnostics.length) break;
  }
  return out;
}

export interface SolveProps {
  id: string;
  starter: string;
  reference: string;
  tests: Test[];
  children?: ReactNode;
}

const key = (id: string) => `cp-solved:${id}`;

export function Solve({ id, starter, reference, tests, children }: SolveProps) {
  const [solved, setSolved] = useState(false);
  const [shown, setShown] = useState(false);
  useEffect(() => setSolved(localStorage.getItem(key(id)) === '1'), [id]);
  const accept = () => {
    localStorage.setItem(key(id), '1');
    setSolved(true);
  };
  return (
    <div className="not-prose space-y-4">
      <CodeTask starter={starter} tests={tests} onSolved={accept} />
      {solved ? (
        <Alert>
          <CheckCircle2 />
          <AlertTitle>Accepted</AlertTitle>
          <AlertDescription>
            <div className="prose prose-sm max-w-none dark:prose-invert [&_ul]:list-disc [&_ul]:pl-5">{children}</div>
          </AlertDescription>
        </Alert>
      ) : (
        <div className="space-y-2">
          {children && (
            <details className="rounded-xl border bg-card px-3 py-2 text-sm">
              <summary className="cursor-pointer font-medium">Hints</summary>
              <div className="prose prose-sm mt-2 max-w-none dark:prose-invert [&_ul]:list-disc [&_ul]:pl-5">{children}</div>
            </details>
          )}
          <Button variant="ghost" size="sm" onClick={() => setShown((s) => !s)}>
            {shown ? 'Hide the reference solution' : 'Show the reference solution'}
          </Button>
        </div>
      )}
      {(solved || shown) && (
        <div className="not-prose overflow-hidden rounded-xl border bg-card shadow-sm">
          <p className="border-b px-3 py-2 text-xs text-muted-foreground">Reference solution</p>
          <CodeEditor value={reference} readOnly />
        </div>
      )}
    </div>
  );
}

function CodeTask({ starter, tests, onSolved }: { starter: string; tests: Test[]; onSolved: () => void }) {
  const [source, setSource] = useState(starter);
  const [busy, setBusy] = useState(false);
  const [verdicts, setVerdicts] = useState<Verdict[] | null>(null);
  const submit = async () => {
    setBusy(true);
    try {
      const v = await judge(source, tests);
      setVerdicts(v);
      if (v.length === tests.length && v.every((x) => x.ok)) onSolved();
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="not-prose overflow-hidden rounded-xl border bg-card shadow-sm">
      <CodeEditor value={source} onChange={setSource} onRun={submit} minHeight="200px" className="border-b" />
      <div className="flex items-center gap-2 p-3">
        <Button size="sm" onClick={submit} disabled={busy}>
          {busy ? <Loader2 className="animate-spin" /> : <Play />} Submit
        </Button>
        <span className="text-sm text-muted-foreground">{tests.length} tests · Ctrl+Enter</span>
      </div>
      {verdicts && <Verdicts verdicts={verdicts} total={tests.length} />}
    </div>
  );
}

function Verdicts({ verdicts, total }: { verdicts: Verdict[]; total: number }) {
  const passed = verdicts.filter((v) => v.ok).length;
  const first = verdicts.find((v) => !v.ok);
  return (
    <div className="space-y-2 border-t p-3 text-sm" role="status">
      <p className="flex items-center gap-2 font-medium">
        {first ? <XCircle className="size-4 text-destructive" /> : <CheckCircle2 className="size-4 text-green-600" />}
        {first ? (first.error ? 'Compile or runtime error' : `Wrong answer on test ${first.test}`) : 'All tests pass'} · {passed}/{total}
      </p>
      {first && (
        <div className="grid gap-2 sm:grid-cols-3">
          <Labeled label="Input" text={first.input} />
          <Labeled label="Expected" text={first.expected} />
          <Labeled label={first.error ? 'Error' : 'Got'} text={first.error ?? first.got} />
        </div>
      )}
    </div>
  );
}

function Labeled({ label, text }: { label: string; text: string }) {
  return (
    <div>
      <p className="mb-1 text-xs text-muted-foreground">{label}</p>
      <pre className="max-h-48 overflow-auto rounded-md bg-muted p-2 font-mono text-xs whitespace-pre-wrap">{text}</pre>
    </div>
  );
}
