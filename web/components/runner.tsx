'use client';

import { Loader2, Play, RotateCcw, Square } from 'lucide-react';
import { useCallback, useState } from 'react';
import { CodeEditor } from '@/components/editor/code-editor';
import { type Status, StatusBadge } from '@/components/status-badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { cpl, stop, type Diagnostic } from '@/lib/cpl';
import { cn } from '@/lib/utils';

export interface RunnerProps {
  source: string;
  stdin?: string;
  status?: Status;
  caption?: string;
  className?: string;
  editorHeight?: string;
  playgroundLink?: boolean;
}

type Result =
  | { kind: 'idle' }
  | { kind: 'running' }
  | { kind: 'output'; stdout: string; stderr: string; ok: boolean; diagnostics: Diagnostic[] }
  | { kind: 'error'; message: string };

export function formatDiagnostics(diagnostics: Diagnostic[], source: string): string {
  return diagnostics
    .map((d) => {
      const line = source.slice(0, d.start).split('\n').length;
      return `${d.severity}[${d.code}]: ${d.message}\n  --> line ${line}${d.help ? `\n  help: ${d.help}` : ''}`;
    })
    .join('\n\n');
}

export function Runner({ source: initial, stdin: initialStdin = '', status, caption, className, editorHeight, playgroundLink = true }: RunnerProps) {
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
    <div className={cn('overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm', className)}>
      <div className="flex items-center gap-2 border-b px-3 py-2">
        <Button size="sm" onClick={run} disabled={result.kind === 'running'}>
          {result.kind === 'running' ? <Loader2 className="animate-spin" /> : <Play />}
          Run
        </Button>
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
          {playgroundLink && (
            <Button size="sm" variant="outline" asChild>
              <a href={`/play#${share}`}>Open in playground</a>
            </Button>
          )}
        </div>
      </div>
      <CodeEditor value={source} onChange={setSource} onRun={run} minHeight={editorHeight} className="border-b" />
      <Tabs value={tab} onValueChange={(v) => (v === 'cpp' ? showCpp() : setTab(v))} className="gap-0">
        <TabsList className="m-2">
          <TabsTrigger value="output">Output</TabsTrigger>
          <TabsTrigger value="input">Input</TabsTrigger>
          <TabsTrigger value="cpp">C++17</TabsTrigger>
        </TabsList>
        <TabsContent value="output" className="px-3 pb-3">
          <Output result={result} source={source} />
        </TabsContent>
        <TabsContent value="input" className="px-3 pb-3">
          <Textarea value={stdin} onChange={(e) => setStdin(e.target.value)} className="min-h-24 font-mono text-xs" spellCheck={false} />
        </TabsContent>
        <TabsContent value="cpp" className="px-3 pb-3">
          <pre className="max-h-80 overflow-auto rounded-md bg-muted p-3 font-mono text-xs">{cpp}</pre>
        </TabsContent>
      </Tabs>
      {caption && <p className="border-t px-3 py-2 text-sm text-muted-foreground">{caption}</p>}
    </div>
  );
}

function Output({ result, source }: { result: Result; source: string }) {
  const box = 'min-h-16 max-h-64 overflow-auto rounded-md bg-muted p-3 font-mono text-xs whitespace-pre-wrap';
  if (result.kind === 'idle') return <pre className={cn(box, 'text-muted-foreground')}>Press Run (Ctrl+Enter).</pre>;
  if (result.kind === 'running') return <pre className={cn(box, 'text-muted-foreground')}>Loading the compiler and running…</pre>;
  if (result.kind === 'error') return <pre className={cn(box, 'text-destructive')}>{result.message}</pre>;
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
