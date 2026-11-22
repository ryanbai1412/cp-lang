'use client';

import { Bug, History, Loader2, Play, RotateCcw, Share2, Square } from 'lucide-react';
import { useCallback, useState } from 'react';
import { CodeEditor } from '@/components/editor/code-editor';
import { type Status, StatusBadge } from '@/components/status-badge';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { cpl, stop, type Diagnostic, type StressOutcome } from '@/lib/cpl';
import { tools } from '@/lib/tools';
import { cn } from '@/lib/utils';

const compilerVersion = tools.version.replace(/^cpl /, '');

export interface RunnerProps {
  source: string;
  stdin?: string;
  status?: Status;
  caption?: string;
  className?: string;
  editorHeight?: string;
  playgroundLink?: boolean;
  shareMenu?: boolean;
  checkOnLoad?: boolean;
}

type Result =
  | { kind: 'idle' }
  | { kind: 'running' }
  | { kind: 'output'; stdout: string; stderr: string; ok: boolean; diagnostics: Diagnostic[] }
  | { kind: 'stress'; outcome: StressOutcome }
  | { kind: 'error'; message: string };

export function formatDiagnostics(diagnostics: Diagnostic[], source: string): string {
  return diagnostics
    .map((d) => {
      const line = source.slice(0, d.start).split('\n').length;
      return `${d.severity}[${d.code}]: ${d.message}\n  --> line ${line}${d.help ? `\n  help: ${d.help}` : ''}`;
    })
    .join('\n\n');
}

export function Runner({ source: initial, stdin: initialStdin = '', status, caption, className, editorHeight, playgroundLink = true,
  shareMenu = false, checkOnLoad = false }: RunnerProps) {
  const [source, setSource] = useState(initial);
  const [stdin, setStdin] = useState(initialStdin);
  const [result, setResult] = useState<Result>({ kind: 'idle' });
  const [cpp, setCpp] = useState<string | null>(null);
  const [tab, setTab] = useState('output');

  const run = useCallback(async () => {
    setTab('output');
    setResult({ kind: 'running' });
    try {
      const r = await cpl.run(source, stdin);
      setResult({ kind: 'output', stdout: r.stdout, stderr: r.stderr, ok: r.ok, diagnostics: r.diagnostics });
    } catch (e) {
      setResult({ kind: 'error', message: (e as Error).message });
    }
  }, [source, stdin]);

  const trace = useCallback(async () => {
    // Opened before the await: a popup opened after it has no user activation and is blocked.
    const viewer = window.open('', '_blank');
    setTab('output');
    setResult({ kind: 'running' });
    try {
      const r = await cpl.trace(source, stdin);
      setResult({ kind: 'output', stdout: r.stdout, stderr: r.stderr, ok: r.ok, diagnostics: r.diagnostics });
      if (r.trace && viewer) {
        viewer.sessionStorage.setItem('cp-live-trace', JSON.stringify(r.trace));
        viewer.sessionStorage.setItem('cp-live-input', stdin);
        viewer.location.href = '/trace.html#live';
      } else viewer?.close();
    } catch (e) {
      viewer?.close();
      setResult({ kind: 'error', message: (e as Error).message });
    }
  }, [source, stdin]);

  const stress = useCallback(async () => {
    setTab('output');
    setResult({ kind: 'running' });
    try {
      setResult({ kind: 'stress', outcome: await cpl.stress(source) });
    } catch (e) {
      setResult({ kind: 'error', message: (e as Error).message });
    }
  }, [source]);
  const stressable = /^@(brute|alt|check|checker)\b/m.test(source);

  const showCpp = useCallback(async () => {
    setTab('cpp');
    setCpp('// Building…');
    try {
      const b = await cpl.build(source);
      setCpp(b.cpp ?? formatDiagnostics(b.diagnostics, source));
    } catch (e) {
      setCpp(`// ${(e as Error).message}`);
    }
  }, [source]);

  const share = typeof window === 'undefined' ? '' : encodeShare(source, stdin);

  return (
    <div className={cn('not-prose overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm', className)}>
      <div className="flex flex-wrap items-center gap-2 border-b px-3 py-2">
        <Button size="sm" onClick={run} disabled={result.kind === 'running'}>
          {result.kind === 'running' ? <Loader2 className="animate-spin" /> : <Play />}
          Run
        </Button>
        <Button size="sm" variant="outline" onClick={trace} disabled={result.kind === 'running'} title="Record every step and open the time-travel debugger">
          <History /> Trace
        </Button>
        {stressable && (
          <Button size="sm" variant="outline" onClick={stress} disabled={result.kind === 'running'} title="Run random cases against the brute force and shrink the first one that differs">
            <Bug /> Stress
          </Button>
        )}
        {result.kind === 'running' && (
          <Button size="sm" variant="ghost" onClick={() => stop().then(() => setResult({ kind: 'idle' }))}>
            <Square /> Stop
          </Button>
        )}
        <Button size="sm" variant="ghost" onClick={() => { setSource(initial); setStdin(initialStdin); }} title="Reset">
          <RotateCcw />
        </Button>
        <div className="ml-auto flex items-center gap-2">
          {status && <StatusBadge status={status} />}
          {shareMenu && <ShareMenu source={source} stdin={stdin} />}
          {playgroundLink && (
            <Button size="sm" variant="outline" asChild>
              <a href={`/play#${share}`} suppressHydrationWarning>Open in playground</a>
            </Button>
          )}
        </div>
      </div>
      <CodeEditor value={source} onChange={setSource} onRun={run} minHeight={editorHeight} className="border-b" checkOnLoad={checkOnLoad} />
      <Tabs value={tab} onValueChange={(v) => (v === 'cpp' ? showCpp() : setTab(v))} className="gap-0">
        <TabsList className="m-2">
          <TabsTrigger value="output">Run</TabsTrigger>
          <TabsTrigger value="cpp">C++17</TabsTrigger>
        </TabsList>
        <TabsContent value="output" className="grid gap-3 px-3 pb-3 sm:grid-cols-2">
          <label className="flex min-w-0 flex-col gap-1 text-xs text-muted-foreground">
            Input
            <Textarea value={stdin} onChange={(e) => setStdin(e.target.value)} className="min-h-16 flex-1 font-mono text-xs text-foreground" spellCheck={false} />
          </label>
          <div className="flex min-w-0 flex-col gap-1 text-xs text-muted-foreground">
            Output
            <Output result={result} source={source} onUseInput={setStdin} />
          </div>
        </TabsContent>
        <TabsContent value="cpp" className="px-3 pb-3">
          <pre className="max-h-80 overflow-auto rounded-md bg-muted p-3 font-mono text-xs">{cpp}</pre>
        </TabsContent>
      </Tabs>
      {caption && <p className="border-t px-3 py-2 text-sm text-muted-foreground">{caption}</p>}
    </div>
  );
}

function Output({ result, source, onUseInput }: { result: Result; source: string; onUseInput: (input: string) => void }) {
  const box = 'min-h-16 max-h-64 overflow-auto rounded-md bg-muted p-3 font-mono text-xs whitespace-pre-wrap';
  if (result.kind === 'idle') return <pre className={cn(box, 'text-muted-foreground')}>Press Run (Ctrl+Enter).</pre>;
  if (result.kind === 'running') return <pre className={cn(box, 'text-muted-foreground')}>Loading the compiler and running…</pre>;
  if (result.kind === 'error') return <pre className={cn(box, 'text-destructive')}>{result.message}</pre>;
  if (result.kind === 'stress') {
    const o = result.outcome;
    if (o.status === 'error') return <pre className={cn(box, 'text-destructive')} role="status">{o.error}</pre>;
    if (o.status === 'ok') return <pre className={box} role="status">{`All ${o.cases} random cases agree.`}</pre>;
    const c = o.counterexample;
    const d = c.explain && 'divergence' in c.explain ? c.explain.divergence : null;
    return (
      <div className="space-y-2 text-sm" role="status">
        <p className="font-medium">
          Case {c.case} of {o.cases} differs ({c.verdict}). Shrunk to the smallest input that still fails:
        </p>
        <div className="grid gap-2 sm:grid-cols-3">
          {[['Input', c.input], ['Expected', c.expected], ['Got', c.got]].map(([label, text]) => (
            <div key={label}>
              <p className="mb-1 text-xs text-muted-foreground">{label}</p>
              <pre className="max-h-48 overflow-auto rounded-md bg-muted p-2 font-mono text-xs whitespace-pre-wrap">{text}</pre>
            </div>
          ))}
        </div>
        {d && (
          <p className="text-muted-foreground">
            First difference: <code>{d.name}</code>
            {d.fast && ` is ${d.fast.value} on line ${d.fast.line}`}
            {d.brute && `, but ${d.brute.value} on line ${d.brute.line} of the brute force`}.
          </p>
        )}
        <Button size="sm" variant="outline" onClick={() => onUseInput(c.input)}>
          Use as input
        </Button>
      </div>
    );
  }
  const errors = result.diagnostics.length ? formatDiagnostics(result.diagnostics, source) : result.stderr;
  return (
    <pre className={box} role="status">
      {result.stdout}
      {errors && <span className={result.ok ? 'text-muted-foreground' : 'text-destructive'}>{(result.stdout ? '\n' : '') + errors}</span>}
    </pre>
  );
}

export function encodeShare(source: string, stdin: string): string {
  const bytes = new TextEncoder().encode(JSON.stringify({ v: 1, src: source, in: stdin }));
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return `c=${btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')}`;
}

export function decodeShare(hash: string): { src: string; in: string } | null {
  const m = /(?:^#?|&)c=([^&]+)/.exec(hash);
  if (!m) return null;
  try {
    const bin = atob(m[1].replace(/-/g, '+').replace(/_/g, '/'));
    const data = JSON.parse(new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0))));
    return { src: String(data.src ?? ''), in: String(data.in ?? '') };
  } catch {
    return null;
  }
}

function ShareMenu({ source, stdin }: { source: string; stdin: string }) {
  const [note, setNote] = useState('');
  const copy = async (url: string, what: string) => {
    await navigator.clipboard.writeText(url);
    history.replaceState(null, '', url);
    setNote(`${what} copied`);
  };
  const short = async () => {
    setNote('Creating…');
    try {
      const r = await fetch('/api/share', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ src: source, in: stdin, version: compilerVersion }),
      });
      if (!r.ok) throw new Error((await r.json().catch(() => ({}))).error ?? r.statusText);
      const { url } = (await r.json()) as { url: string };
      await copy(new URL(url, location.origin).toString(), 'Short link');
    } catch (e) {
      setNote(`Short links aren't available here (${(e as Error).message})`);
    }
  };
  return (
    <>
      {note && (
        <span className="text-xs text-muted-foreground" aria-live="polite">
          {note}
        </span>
      )}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm" variant="outline">
            <Share2 /> Share
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onSelect={() => copy(`${location.origin}/play#${encodeShare(source, stdin)}`, 'Link')}>
            Copy link (stays in the URL)
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={short}>Create short link (for chat)</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  );
}
