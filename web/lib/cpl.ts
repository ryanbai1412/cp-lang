'use client';

import type { BuildOutcome, ExplainOutcome, Outcome, RunOutcome, SampleOptions, StressOptions, StressOutcome, TraceOutcome } from './cpl-types';

export type * from './cpl-types';

type Worker = {
  check(source: string): Promise<Outcome>;
  lint(source: string): Promise<Outcome>;
  run(source: string, stdin?: string): Promise<RunOutcome>;
  trace(source: string, stdin?: string): Promise<TraceOutcome>;
  stress(source: string, options?: StressOptions): Promise<StressOutcome>;
  sample(source: string, options?: SampleOptions): Promise<{ input: string } | { error: string }>;
  build(source: string): Promise<BuildOutcome>;
  explain(source: string): Promise<ExplainOutcome>;
  lsp(message: object): Promise<unknown[]>;
  terminate(): void;
};

let worker: Promise<Worker> | null = null;

async function spawnWorker(): Promise<Worker> {
  const mod = await import(/* webpackIgnore: true */ `${location.origin}/wasm/cpl.js`);
  return mod.spawn({ timeoutMs: 10_000 }) as Worker;
}

/** Whether the compiler has been requested yet; loading it fetches cpl.wasm. */
export const started = () => worker !== null;

export function compiler(): Promise<Worker> {
  worker ??= spawnWorker();
  return worker;
}

export async function stop() {
  if (!worker) return;
  (await worker).terminate();
  worker = null;
}

async function call<T>(f: (w: Worker) => Promise<T>): Promise<T> {
  try {
    return await f(await compiler());
  } catch (error) {
    const kind = (error as { kind?: string }).kind;
    // A worker that couldn't fetch its wasm is dropped too, so the next call
    // retries the download (site/R17).
    const unloaded = kind === 'load' && !/is in cpl-build\.wasm/.test((error as Error).message);
    if (kind === 'timeout' || kind === 'terminated' || unloaded) worker = null;
    throw error;
  }
}

export const cpl = {
  check: (src: string) => call((w) => w.check(src)),
  lint: (src: string) => call((w) => w.lint(src)),
  run: (src: string, stdin = '') => call((w) => w.run(src, stdin)),
  trace: (src: string, stdin = '') => call((w) => w.trace(src, stdin)),
  stress: (src: string, options?: StressOptions) => call((w) => w.stress(src, options)),
  sample: (src: string, options?: SampleOptions) => call((w) => w.sample(src, options)),
  build: (src: string) => call((w) => w.build(src)),
  explain: (src: string) => call((w) => w.explain(src)),
  lsp: (message: object) => call((w) => w.lsp(message)),
};
