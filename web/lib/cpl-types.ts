// Types for cpl.js. Offsets (`start`, `end`, the `offset` arguments) index
// the source as a JavaScript string; `range` is the same span as a Language
// Server Protocol range, with zero-based lines and characters.

export interface Position {
  line: number;
  character: number;
}

export interface Located {
  start: number;
  end: number;
  range: { start: Position; end: Position };
}

export interface Diagnostic extends Located {
  /** Stable code: `E0301`, or `R0001` for a run that failed. */
  code: string;
  severity: "error" | "warning" | "note";
  message: string;
  labels: (Located & { message: string })[];
  notes: string[];
  help: string | null;
  /** Edits that fix it, each replacing `start..end` with `text`. */
  fixes: { title: string; edits: (Located & { text: string })[] }[];
}

export interface Outcome {
  ok: boolean;
  /** As `cpl` exits: 0, 1 when the source has errors, 3 when the run failed. */
  exitCode: 0 | 1 | 3;
  diagnostics: Diagnostic[];
  /** What `cpl` prints on standard error. */
  stderr: string;
  /** Std modules it lacked, when it lacked any; `stdNeeded` names them all. */
  needs?: string[];
}

export interface RunOutcome extends Outcome {
  /** What the program printed, read as UTF-8. */
  stdout: string;
  stdoutBytes: Uint8Array;
  /** Bytes of stack the program's calls could take in this run. */
  maxStack: number;
}

export interface TraceOutcome extends RunOutcome {
  /** A `tools/trace/trace-schema.json` document, or null. */
  trace: { schemaVersion: 1; runs: { fast: object; slow?: object }; [key: string]: unknown } | null;
  /** What the program's `debug(...)`s showed, a line each. */
  debug: string;
}

export interface ExplainOutcome extends Outcome {
  /** What `cpl explain <file>` prints, or null when the source doesn't compile. */
  text: string | null;
}

export interface BuildOutcome extends Outcome {
  /** The C++17 file, or null when there is none. */
  cpp: string | null;
  /** Its size in bytes, against a judge's 64 KB limit. */
  bytes: number;
}

export interface Hover extends Located {
  text: string;
}

export interface Symbol extends Located {
  name: string;
  kind: "value" | "fn" | "struct" | "enum" | "variant" | "domain" | "alias" | "trait" | "impl";
  /** The name itself, inside the declaration. */
  selection: Located;
}

export interface SampleOptions extends Options {
  /** Default 1. */
  seed?: number;
  /** Maximum size (`n`, `q`, ...) generated. Default 8. */
  size?: number;
}

export interface StressOptions extends Options {
  /** How many cases to run. Default 200. */
  cases?: number;
  /** Starting seed; case `i` uses `seed + i`. Default 1. */
  seed?: number;
  /** Maximum size (`n`, `q`, ...) generated. Default 8. */
  size?: number;
}

export interface StressStage {
  name: string;
  line: number;
  fast: string;
  brute: string;
}

export interface StressDivergence {
  name: string;
  nth: number;
  fast: { line: number; value: string } | null;
  brute: { line: number; value: string } | null;
}

export interface StressCounterexample {
  case: number;
  seed: number;
  /** Human-readable input with the program's names (`n = 2  a = [1, 1]`). */
  input: string;
  /** Shrunk counterexample in input format. */
  raw_input: string;
  expected: string;
  got: string;
  verdict: "WA" | "RE" | "TLE" | "MLE";
  stages: StressStage[];
  saved: string | null;
  trace: string | null;
  explain?: { divergence: StressDivergence | null } | { error: string };
}

export type StressOutcome =
  | { status: "ok"; cases: number }
  | { status: "fail"; cases: number; counterexample: StressCounterexample }
  | { status: "error"; error: string };

/** One call replayed with its fn edited; values are trace JSON. */
export interface ReplayOutcome {
  call: { name: string; ordinal: number; entry: number; exit: number; args: Record<string, unknown> };
  /** Why the whole file ran instead of the one call, or null. */
  fallback: string | null;
  /** False when the edited run never made the call. */
  replayed: boolean;
  returned: { before: unknown; after: unknown };
  /** Each top-level variable the call changed, by id. */
  writes: { id: string; before: unknown; after: unknown }[];
  /** The first step of the replayed call that differs; `variable` is null when one run ended first. */
  change: { step: number; line: number; variable: string | null; before: unknown; after: unknown } | null;
  /** A step before the call that already differs: the edit changes an earlier call. */
  earlier: { step: number; line: number } | null;
  /** The message of the trap that stopped the replay, or null. */
  trap: string | null;
}

export interface Options {
  /** The file's name in diagnostics and in the C++ header. Default `main.cp`. */
  name?: string;
}

export declare class CplError extends Error {
  /**
   * `stack` and `memory`: the browser stopped the program. `timeout` and
   * `terminated`: the page stopped the worker. `bug`: the compiler failed.
   * `load`: the module or the worker could not be loaded.
   */
  kind: "stack" | "memory" | "timeout" | "terminated" | "bug" | "load";
}

export declare class Cpl {
  readonly version: string;
  check(source: string, options?: Options): Outcome;
  /** `cpl lint`: the lint warnings in `source` at default levels, with their fixes. */
  lint(source: string, options?: Options): Outcome;
  /** `cpl fix`: `source` with the fixes `cpl fix` applies, as `fixed`, and the errors left. */
  fix(source: string, options?: Options): Outcome & { fixed: string; applied: number };
  run(source: string, stdin?: string | Uint8Array, options?: Options): RunOutcome;
  /**
   * `cpl run --trace`: the run, recorded. `trace` is a document of
   * `tools/trace/trace-schema.json` for the trace viewer, null when the
   * source has errors; with a `@alt(program)` fn in the file it has the
   * brute's run too, as `runs.slow`. `debug` is what `debug(...)` showed.
   */
  trace(source: string, stdin?: string | Uint8Array, options?: Options): TraceOutcome;
  /**
   * `cpl replay`: the call open at `step` (0-based) of the run of `original`
   * on `stdin`, `frame` callers out, run again with the fns of `edited`.
   */
  replay(
    original: string,
    edited: string,
    stdin?: string | Uint8Array,
    step?: number,
    frame?: number,
    options?: Options,
  ): ReplayOutcome | { error: string };
  /**
   * `cpl stress --explain --format json`: generates inputs, compares against
   * `@alt(program)` / `@alt(program)` / `@check` / `@check`, and shrinks the
   * first counterexample with the same Rust shrinker as the CLI.
   */
  stress(source: string, options?: StressOptions): StressOutcome;
  build(source: string, options?: Options): BuildOutcome;
  explain(source: string, options?: Options): ExplainOutcome;
  /** A random input for `source` from its `read()` types, as `cpl stress` makes one. */
  sample(source: string, options?: SampleOptions): { input: string } | { error: string };
  hover(source: string, offset: number, options?: Options): Hover | null;
  definition(source: string, offset: number, options?: Options): Located | null;
  symbols(source: string, options?: Options): Symbol[];
  lsp(message: object): object[];
  /** The std modules `source` needs that this compiler lacks; ask again after giving them. */
  stdNeeded(source: string, options?: Options): string[];
  /** Gives the compiler `std/<module>.cp`; giving it again is no change. */
  provideStd(module: string, text: string): { changed: boolean } | { error: string };
}

export type Worker = {
  [Method in "stdNeeded" | "provideStd" | "check" | "lint" | "run" | "trace" | "replay" | "stress" | "sample" | "build" | "explain" | "hover" | "definition" | "symbols" | "lsp"]: (
    ...args: Parameters<Cpl[Method]>
  ) => Promise<ReturnType<Cpl[Method]>>;
} & { terminate(): void };

/** Where `cpl-build.wasm`, the module with `build`, `explain`, `stress` and `sample`, is. */
export declare const BUILD_WASM: URL;
/** The methods only `cpl-build.wasm` has; on `cpl.wasm` they throw a `CplError` of kind "load". */
export declare const BUILD_CALLS: readonly ["stress", "sample", "build", "explain"];
export declare function load(wasm?: string | URL | BufferSource | WebAssembly.Module): Promise<Cpl>;
export declare function spawn(options?: { timeoutMs?: number }): Worker;
