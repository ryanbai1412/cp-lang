// CP Lang in the browser: the compiler, the debug interpreter and the C++17
// back end as one WebAssembly module, with no server.
//
//   import { load } from "./cpl.js";
//   const cpl = await load();
//   cpl.run("let n: Int = read()\nprint(n * 2)\n", "21").stdout   // "42\n"
//
// `load()` runs the compiler on the calling thread. `spawn()` runs it in a
// Web Worker, which is what a page wants for running a program: the page
// stays responsive, and a program that never ends is stopped after a time
// limit. crates/cpl-wasm/README.md in the cp-lang repository has the full
// interface; cpl.d.ts has its types.

// Slots of the module's interface (crates/cpl-wasm/src/abi.rs).
const NAME = 0, TEXT = 1, STDIN = 2;
const JSON_OUT = 0, STDOUT = 1, PANIC = 2;

// Below this many nested calls, a run that overflows the stack is not the
// program's recursion: `run` stops trying.
const MIN_DEPTH = 16;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** The compiler itself failed, or the browser stopped it. `kind` says which. */
export class CplError extends Error {
  constructor(kind, message) {
    super(message);
    this.name = "CplError";
    this.kind = kind;
  }
}

/**
 * Loads the compiler. `wasm` is where `cpl.wasm` is (a URL, by default next
 * to this file), or its bytes, or a compiled `WebAssembly.Module`.
 */
export async function load(wasm = new URL("./cpl.wasm", import.meta.url)) {
  const module = await compile(wasm);
  return new Cpl(module, await WebAssembly.instantiate(module, {}));
}

async function compile(wasm) {
  if (wasm instanceof WebAssembly.Module) return wasm;
  if (wasm instanceof ArrayBuffer || ArrayBuffer.isView(wasm)) return WebAssembly.compile(wasm);
  const url = new URL(wasm, import.meta.url);
  if (url.protocol === "file:") {
    // Node: `fetch` does not read files.
    const { readFile } = await import("node:fs/promises");
    return WebAssembly.compile(await readFile(url));
  }
  const response = await fetch(url);
  if (!response.ok) throw new CplError("load", `cpl: can't load ${url}: HTTP ${response.status}`);
  const type = response.headers.get("content-type") ?? "";
  // Streaming compilation needs the server to say `application/wasm`.
  return type.startsWith("application/wasm")
    ? WebAssembly.compileStreaming(response)
    : WebAssembly.compile(await response.arrayBuffer());
}

/** The compiler, running on the calling thread. Every method is synchronous. */
export class Cpl {
  #module;
  #exports;
  #maxDepth;

  constructor(module, instance) {
    this.#module = module;
    this.#exports = instance.exports;
    const { version, maxDepth } = this.#call("cpl_version", {});
    this.version = version;
    this.#maxDepth = maxDepth;
  }

  /** `cpl check`: every diagnostic in `source`. */
  check(source, options = {}) {
    return this.#call("cpl_check", { ...options, source });
  }

  /** `cpl run`: checks `source`, then runs it with `stdin` as its input. */
  run(source, stdin = "", options = {}) {
    // The program's calls are the interpreter's, on the browser's stack,
    // and the module cannot see how much of it is left. A run that
    // overflows it is run again with the limit on nested calls halved, until
    // the interpreter stops it first: that is an error with a source span,
    // where the overflow is only a trap. `maxDepth` is the limit it ran with.
    for (let maxDepth = this.#maxDepth; ; maxDepth >>= 1) {
      try {
        return { ...this.#call("cpl_run", { ...options, source, stdin }, maxDepth), maxDepth };
      } catch (error) {
        if (error.kind !== "stack" || maxDepth < MIN_DEPTH) throw error;
        // Not the program's calls if the front end overflows by itself:
        // this throws, and no limit would help.
        if (maxDepth === this.#maxDepth) this.check(source, options);
      }
    }
  }

  /** `cpl build`: checks `source`, then turns it into one C++17 file. */
  build(source, options = {}) {
    const result = this.#call("cpl_build", { ...options, source });
    result.bytes = result.cpp === null ? 0 : encoder.encode(result.cpp).length;
    return result;
  }

  /** The type and cost of the name at `offset` in `source`, or null. */
  hover(source, offset, options = {}) {
    return this.#call("cpl_hover", { ...options, source }, offset);
  }

  /** Where the name at `offset` in `source` is declared, or null. */
  definition(source, offset, options = {}) {
    return this.#call("cpl_definition", { ...options, source }, offset);
  }

  /** The top-level declarations of `source`, in source order. */
  symbols(source, options = {}) {
    return this.#call("cpl_symbols", { ...options, source });
  }

  /**
   * Hands one Language Server Protocol message (an object) to the language
   * server and returns the messages it sends back.
   */
  lsp(message) {
    return this.#call("cpl_lsp", { source: JSON.stringify(message) });
  }

  #call(command, { name = "main.cp", source = "", stdin = "" }, ...args) {
    const exports = this.#exports;
    try {
      this.#write(NAME, encoder.encode(name));
      this.#write(TEXT, encoder.encode(source));
      this.#write(STDIN, typeof stdin === "string" ? encoder.encode(stdin) : stdin);
      exports[command](...args);
      const result = JSON.parse(decoder.decode(this.#read(JSON_OUT)));
      if (command === "cpl_run") {
        // A program prints bytes. `stdout` is those bytes read as UTF-8.
        result.stdoutBytes = this.#read(STDOUT).slice();
        result.stdout = decoder.decode(result.stdoutBytes);
      }
      return result;
    } catch (error) {
      throw this.#restart(error);
    }
  }

  #write(slot, bytes) {
    const at = this.#exports.cpl_input(slot, bytes.length);
    new Uint8Array(this.#exports.memory.buffer, at, bytes.length).set(bytes);
  }

  // A view, good until the next call: the memory may move when it grows.
  #read(slot) {
    const { memory, cpl_output_ptr, cpl_output_len } = this.#exports;
    return new Uint8Array(memory.buffer, cpl_output_ptr(slot), cpl_output_len(slot));
  }

  // A call that ends in a trap leaves the module half-way through it, so the
  // module is replaced; the next call starts clean.
  #restart(error) {
    let panic = "";
    try {
      panic = decoder.decode(this.#read(PANIC));
    } catch {
      // The module is too far gone to say; the error below stands alone.
    }
    this.#exports = new WebAssembly.Instance(this.#module, {}).exports;
    if (panic) {
      return new CplError("bug", `cpl: internal error, this is a bug in cpl: ${panic}`);
    }
    if (error instanceof RangeError) {
      return new CplError(
        "stack",
        "cpl: ran out of stack: the browser allows less nesting than `cpl run` on your machine",
      );
    }
    return new CplError(
      "memory",
      `cpl: stopped by the browser, most likely out of memory (${error?.message ?? error})`,
    );
  }
}

const METHODS = ["check", "run", "build", "hover", "definition", "symbols", "lsp"];

/**
 * The compiler in a Web Worker: the same methods as `Cpl`, each returning a
 * promise. A call that takes longer than `timeoutMs` stops the worker and
 * rejects with a `CplError` of kind "timeout"; the next call starts a new
 * worker. Browsers only.
 */
export function spawn({ timeoutMs = 10_000 } = {}) {
  let worker = null;
  let nextId = 0;
  const pending = new Map();

  const stop = (error) => {
    worker?.terminate();
    worker = null;
    for (const { reject, timer } of pending.values()) {
      clearTimeout(timer);
      reject(error);
    }
    pending.clear();
  };

  const start = () => {
    worker = new Worker(new URL("./cpl.worker.js", import.meta.url), { type: "module" });
    worker.onmessage = ({ data: { id, result, error } }) => {
      const call = pending.get(id);
      if (!call) return;
      pending.delete(id);
      clearTimeout(call.timer);
      if (error) call.reject(new CplError(error.kind, error.message));
      else call.resolve(result);
    };
    worker.onerror = (event) =>
      stop(new CplError("load", `cpl: the worker failed: ${event.message ?? "could not start"}`));
  };

  const call = (method, args) =>
    new Promise((resolve, reject) => {
      if (!worker) start();
      const id = nextId++;
      const seconds = (timeoutMs / 1000).toFixed(timeoutMs % 1000 ? 1 : 0);
      const timer = setTimeout(
        () => stop(new CplError("timeout", `cpl: stopped after ${seconds} s`)),
        timeoutMs,
      );
      pending.set(id, { resolve, reject, timer });
      worker.postMessage({ id, method, args });
    });

  const client = {
    /** Stops the worker. Calls still running reject; a later call starts a new one. */
    terminate: () => stop(new CplError("terminated", "cpl: the worker was stopped")),
  };
  for (const method of METHODS) client[method] = (...args) => call(method, args);
  return client;
}
