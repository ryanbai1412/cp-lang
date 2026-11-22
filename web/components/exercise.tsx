'use client';

import { CheckCircle2, Lightbulb, Loader2, Play, XCircle } from 'lucide-react';
import { useState } from 'react';
import { CodeEditor } from '@/components/editor/code-editor';
import { formatDiagnostics } from '@/components/runner';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { cpl } from '@/lib/cpl';
import type { Exercise as Ex, Test } from '@/lib/course';
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

export function Exercise({ exercise }: { exercise: Ex }) {
  const [hint, setHint] = useState(false);
  const [solved, setSolved] = useState(false);
  return (
    <div className="space-y-4">
      <p className="text-base leading-7">{exercise.prompt}</p>
      {exercise.kind === 'code' && <CodeTask exercise={exercise} onSolved={() => setSolved(true)} />}
      {exercise.kind === 'output' && <OutputTask exercise={exercise} onSolved={() => setSolved(true)} />}
      {exercise.kind === 'choice' && <ChoiceTask exercise={exercise} onSolved={() => setSolved(true)} />}
      <div className="flex gap-2">
        <Button variant="ghost" size="sm" onClick={() => setHint((h) => !h)}>
          <Lightbulb /> {hint ? 'Hide hint' : 'Hint'}
        </Button>
      </div>
      {hint && (
        <Alert>
          <Lightbulb />
          <AlertTitle>Hint</AlertTitle>
          <AlertDescription>{exercise.hint}</AlertDescription>
        </Alert>
      )}
      {solved && (
        <Alert>
          <CheckCircle2 />
          <AlertTitle>Accepted</AlertTitle>
          <AlertDescription>{exercise.solution}</AlertDescription>
        </Alert>
      )}
    </div>
  );
}

function CodeTask({ exercise, onSolved }: { exercise: Extract<Ex, { kind: 'code' }>; onSolved: () => void }) {
  const [source, setSource] = useState(exercise.starter);
  const [busy, setBusy] = useState(false);
  const [verdicts, setVerdicts] = useState<Verdict[] | null>(null);
  const submit = async () => {
    setBusy(true);
    try {
      const v = await judge(source, exercise.tests);
      setVerdicts(v);
      if (v.length === exercise.tests.length && v.every((x) => x.ok)) onSolved();
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
      <CodeEditor value={source} onChange={setSource} onRun={submit} minHeight="200px" className="border-b" />
      <div className="flex items-center gap-2 p-3">
        <Button size="sm" onClick={submit} disabled={busy}>
          {busy ? <Loader2 className="animate-spin" /> : <Play />} Submit
        </Button>
        <span className="text-sm text-muted-foreground">{exercise.tests.length} tests · Ctrl+Enter</span>
      </div>
      {verdicts && <Verdicts verdicts={verdicts} total={exercise.tests.length} />}
    </div>
  );
}

function OutputTask({ exercise, onSolved }: { exercise: Extract<Ex, { kind: 'output' }>; onSolved: () => void }) {
  const [answers, setAnswers] = useState(exercise.tests.map(() => ''));
  const [checked, setChecked] = useState<boolean[] | null>(null);
  const check = () => {
    const c = exercise.tests.map((t, i) => norm(answers[i]) === norm(t.expected));
    setChecked(c);
    if (c.every(Boolean)) onSolved();
  };
  return (
    <div className="space-y-3">
      <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
        <CodeEditor value={exercise.code} readOnly />
      </div>
      {exercise.tests.map((t, i) => (
        <div key={i} className="grid gap-2 sm:grid-cols-2">
          <pre className="rounded-md bg-muted p-3 font-mono text-xs">{t.input}</pre>
          <textarea
            aria-label={`Predicted output ${i + 1}`}
            className={cn(
              'min-h-16 rounded-md border bg-background p-3 font-mono text-xs',
              checked && (checked[i] ? 'border-green-600' : 'border-destructive'),
            )}
            placeholder="Predicted output"
            value={answers[i]}
            onChange={(e) => setAnswers((a) => a.map((x, j) => (j === i ? e.target.value : x)))}
          />
        </div>
      ))}
      <Button size="sm" onClick={check}>Check</Button>
    </div>
  );
}

function ChoiceTask({ exercise, onSolved }: { exercise: Extract<Ex, { kind: 'choice' }>; onSolved: () => void }) {
  const [picked, setPicked] = useState<number | null>(null);
  return (
    <div className="space-y-3">
      {exercise.code && (
        <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
          <CodeEditor value={exercise.code} readOnly />
        </div>
      )}
      <div className="grid gap-2">
        {exercise.options.map((o, i) => (
          <Button
            key={i}
            variant="outline"
            className={cn(
              'h-auto justify-start whitespace-normal py-3 text-left',
              picked === i && (i === exercise.answer ? 'border-green-600' : 'border-destructive'),
            )}
            onClick={() => {
              setPicked(i);
              if (i === exercise.answer) onSolved();
            }}
          >
            {o}
          </Button>
        ))}
      </div>
      {picked !== null && picked !== exercise.answer && (
        <p className="text-sm text-destructive">Not quite. Try another option.</p>
      )}
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
