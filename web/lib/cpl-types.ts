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

export interface Options {
  /** The file's name in diagnostics and in the C++ header. Default `main.cp`. */
  name?: string;
}

export interface CplError extends Error {
  /**
   * `stack` and `memory`: the browser stopped the program. `timeout` and
   * `terminated`: the page stopped the worker. `bug`: the compiler failed.
   * `load`: the module or the worker could not be loaded.
   */
  kind: "stack" | "memory" | "timeout" | "terminated" | "bug" | "load";
}

export interface Cpl {
  readonly version: string;
  check(source: string, options?: Options): Outcome;
  run(source: string, stdin?: string | Uint8Array, options?: Options): RunOutcome;
  /**
   * `cpl run --trace`: the run, recorded. `trace` is a document of
   * `tools/trace/trace-schema.json` for the trace viewer, null when the
   * source has errors; with a `@brute(main)` fn in the file it has the
   * brute's run too, as `runs.slow`. `debug` is what `debug(...)` showed.
   */
  trace(source: string, stdin?: string | Uint8Array, options?: Options): TraceOutcome;
  build(source: string, options?: Options): BuildOutcome;
  hover(source: string, offset: number, options?: Options): Hover | null;
  definition(source: string, offset: number, options?: Options): Located | null;
  symbols(source: string, options?: Options): Symbol[];
  lsp(message: object): object[];
}

export type Worker = {
  [Method in "check" | "run" | "trace" | "build" | "hover" | "definition" | "symbols" | "lsp"]: (
    ...args: Parameters<Cpl[Method]>
  ) => Promise<ReturnType<Cpl[Method]>>;
} & { terminate(): void };

