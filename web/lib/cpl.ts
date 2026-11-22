'use client';

import type { BuildOutcome, Outcome, RunOutcome, TraceOutcome } from './cpl-types';

export type * from './cpl-types';

type Worker = {
  check(source: string): Promise<Outcome>;
  run(source: string, stdin?: string): Promise<RunOutcome>;
  trace(source: string, stdin?: string): Promise<TraceOutcome>;
  build(source: string): Promise<BuildOutcome>;
  terminate(): void;
};

let worker: Promise<Worker> | null = null;

async function spawnWorker(): Promise<Worker> {
  const mod = await import(/* webpackIgnore: true */ `${location.origin}/wasm/cpl.js`);
  return mod.spawn({ timeoutMs: 10_000 }) as Worker;
}

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
    if (kind === 'timeout' || kind === 'terminated') worker = null;
    throw error;
  }
}

export const cpl = {
  check: (src: string) => call((w) => w.check(src)),
  run: (src: string, stdin = '') => call((w) => w.run(src, stdin)),
  trace: (src: string, stdin = '') => call((w) => w.trace(src, stdin)),
  build: (src: string) => call((w) => w.build(src)),
};
