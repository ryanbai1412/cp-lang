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
  stdNeeded?(source: string): Promise<string[]>;
  provideStd?(module: string, text: string): Promise<{ changed: boolean } | { error: string }>;
  terminate(): void;
};

let worker: Promise<Worker> | null = null;

async function spawnWorker(): Promise<Worker> {
  const mod = await import(/* webpackIgnore: true */ `${location.origin}/wasm/cpl.js`);
  return mod.spawn({ timeoutMs: 10_000 }) as Worker;
}

/** Whether the compiler has been requested yet; loading it fetches cpl.wasm. */
export const started = () => worker !== null;

let state: 'idle' | 'loading' | 'ready' | 'failed' = 'idle';
/** Whether the compiler has answered, is still loading, or couldn't fetch cpl.wasm. */
export const compilerState = () => state;

export function compiler(): Promise<Worker> {
  worker ??= spawnWorker();
  return worker;
}

export async function stop() {
  if (!worker) return;
  (await worker).terminate();
  worker = null;
}

// cpl.wasm carries only std's outline; a module's text is fetched the first
// time a program needs it, then given to every later worker by cpl.js.
const stdTexts = new Map<string, Promise<string>>();

function stdText(module: string): Promise<string> {
  let text = stdTexts.get(module);
  if (!text) {
    text = fetch(`${location.origin}/wasm/std/${module}.cp`).then((r) => {
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      return r.text();
    });
    text.catch(() => stdTexts.delete(module));
    stdTexts.set(module, text);
  }
  return text;
}

export async function provideStd(w: Worker, source: string | undefined) {
  if (source === undefined || !w.stdNeeded || !w.provideStd) return;
  for (let need = await w.stdNeeded(source); need.length; need = await w.stdNeeded(source)) {
    await Promise.all(
      need.map(async (module) => {
        let text: string;
        try {
          text = await stdText(module);
        } catch (error) {
          throw Object.assign(
            new Error(`cpl: can't load std.${module}: ${(error as Error).message}; check the connection and try again`),
            { kind: 'load' },
          );
        }
        const given = await w.provideStd!(module, text);
        if ('error' in given) throw Object.assign(new Error(given.error), { kind: 'bug' });
      }),
    );
  }
}

async function call<T>(f: (w: Worker) => Promise<T>, source?: string): Promise<T> {
  if (state !== 'ready') state = 'loading';
  try {
    const w = await compiler();
    await provideStd(w, source);
    const result = await f(w);
    state = 'ready';
    return result;
  } catch (error) {
    const kind = (error as { kind?: string }).kind;
    // A worker that couldn't fetch its wasm is dropped too, so the next call
    // retries the download (site/R17).
    const unloaded = kind === 'load' && !/is in cpl-build\.wasm/.test((error as Error).message);
    if (kind === 'timeout' || kind === 'terminated' || unloaded) worker = null;
    state = unloaded ? 'failed' : kind === 'timeout' || kind === 'terminated' ? 'idle' : 'ready';
    throw error;
  }
}

/** The whole text an LSP notification opens or changes a document to. */
export function documentText(message: object): string | undefined {
  const params = (message as { params?: { textDocument?: { text?: string }; contentChanges?: { text?: string }[] } }).params;
  return params?.textDocument?.text ?? params?.contentChanges?.at(-1)?.text;
}

export const cpl = {
  check: (src: string) => call((w) => w.check(src), src),
  lint: (src: string) => call((w) => w.lint(src), src),
  run: (src: string, stdin = '') => call((w) => w.run(src, stdin), src),
  trace: (src: string, stdin = '') => call((w) => w.trace(src, stdin), src),
  stress: (src: string, options?: StressOptions) => call((w) => w.stress(src, options), src),
  sample: (src: string, options?: SampleOptions) => call((w) => w.sample(src, options), src),
  build: (src: string) => call((w) => w.build(src), src),
  explain: (src: string) => call((w) => w.explain(src), src),
  lsp: (message: object) => call((w) => w.lsp(message), documentText(message)),
};
