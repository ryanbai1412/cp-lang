# cpl-wasm

The compiler as a WebAssembly module. The docs site and the playground use
it to check, run and build CP Lang in the browser, with no server: the
front end (`cpl-syntax`, `cpl-check`), the typed lowering (`cpl-ir`), the
debug interpreter (`cpl-interp`), the C++17 back end (`cpl-codegen`) and
the language server (`cpl-lsp`) in one `.wasm` file of about 1 MB
(0.4 MB gzipped), and one small JavaScript file that loads it.

```js
import { load } from "./cpl.js";

const cpl = await load();

const run = cpl.run("let n: Int = read()\nprint(n * 2)\n", "21");
run.stdout;                    // "42\n"

const check = cpl.check("print(total)\n");
check.diagnostics[0].message;  // "no `total` in scope"
check.diagnostics[0].range;    // { start: { line: 0, character: 6 }, end: ... }
check.stderr;                  // the same error as `cpl check` prints it

const build = cpl.build("let n: Int = read()\nprint(n * 2)\n");
build.cpp;                     // "// main.cp: C++17 written by cpl 0.0.0 ..."
```

It is the same code as the `cpl` binary, through the same
`cpl_check::Database`, so a program means in the browser what it means on
the command line. `tests/parity.rs` holds the two to the same bytes.

## Building

```text
$ rustup target add wasm32-unknown-unknown    # once
$ scripts/build-wasm.sh
    3094 bytes  dist/wasm/cpl.d.ts
    8892 bytes  dist/wasm/cpl.js
 1078208 bytes  dist/wasm/cpl.wasm
     679 bytes  dist/wasm/cpl.worker.js
  381546 bytes  cpl.wasm, gzipped
```

`dist/` is not tracked: nothing built is ever committed. The build needs
only `cargo` and the `wasm32-unknown-unknown` target. There is no
`wasm-bindgen` or `wasm-pack` step, because the module's interface is a
dozen plain functions (`src/abi.rs`) and `js/cpl.js` is written by hand
against them. If `wasm-opt` is installed the script runs it, and the file
gets a little smaller; without it the build is the same otherwise.

`scripts/build-wasm.sh --out DIR` builds into another directory, and
`--smoke` runs the Node test below after building.

A page needs the files of `dist/wasm/` side by side, served from
anywhere static. `cpl.js` finds `cpl.wasm` next to itself.

## The interface

`load()` gives the compiler on the calling thread, and every method
answers at once. `check`, `run` and `build` are the three commands of
`cpl`, and each result has the same four fields:

```js
{
  ok: false,
  exitCode: 1,            // as `cpl` exits: 0, 1 for errors, 3 for a failed run
  diagnostics: [ ... ],   // for an editor
  stderr: "error[E0301]: ...\n  --> main.cp:4:9\n ...",   // for a console
}
```

`run` adds `stdout` (and `stdoutBytes`, since a program prints bytes) and
`maxStack` (see below), and `build` adds `cpp` and its size in `bytes`. A run that fails keeps what it
printed, exits with 3, and has the failure as its last diagnostic, with
the span of the expression that failed.

A diagnostic says where it is twice. `start` and `end` index the source
string you passed in, in UTF-16 code units like every JavaScript string
index, so `source.slice(d.start, d.end)` is the text it points at.
`range` is the same span as a Language Server Protocol range, which is
what CodeMirror's and Monaco's lint interfaces take:

```js
{
  start: 70, end: 71,
  range: { start: { line: 3, character: 8 }, end: { line: 3, character: 9 } },
  code: "E0301", severity: "error",
  message: "`a` is indexed by V, but `n` is a 1..=9",
  help: "convert it with `V(n)`, or write `a.raw[n]` to index by number",
  labels: [], notes: [], fixes: [],
}
```

For the editor there are also `hover(source, offset)`,
`definition(source, offset)` and `symbols(source)`, and `lsp(message)`,
which hands one protocol message to the server behind `cpl lsp` and
returns the messages it sends back, for an editor component that already
speaks the protocol. Every method takes `{ name: "sum.cp" }` as a last
argument to name the file in diagnostics and in the C++ header.
`js/cpl.d.ts` has the types of all of it.

The module keeps one incremental database between calls, so checking a
file again after a keystroke redoes only the items the edit touched.

## Running a program without freezing the page

A program can loop forever, and WebAssembly cannot be interrupted from
the thread it runs on. `spawn()` gives the same methods from a Web
Worker, each returning a promise:

```js
import { spawn } from "./cpl.js";

const cpl = spawn({ timeoutMs: 5000 });
try {
  const run = await cpl.run(source, stdin);
} catch (error) {
  error.kind;      // "timeout"
  error.message;   // "cpl: stopped after 5 s"
}
```

A call over the time limit stops the worker, and the next call starts a
new one. A page that only checks and builds can use `load()` directly: a
check is a few milliseconds.

## What is different from `cpl run`

One thing: how much stack calls can take. `cpl run` gives a program a stack of
its own, deep enough for a depth-first search over a few million
nodes. A browser gives WebAssembly about 1 MB, and no page can ask for
more. That holds a few hundred to a thousand nested calls, depending on
the program and on the browser. Past that, a run in the browser ends with
the error `cpl run` gives for recursion that never ends:

```text
error[R0001]: the stack ran out: `visit` never stops calling itself?
  --> main.cp:16:26
   |
16 |                     0 => visit(v)
   |                          ^^^^^^^^
```

The module stops a run when its calls take 3 MB of the module's stack;
nothing counts how deep they nest. It cannot see how much of the
browser's own stack is left, so a program with heavier calls can overflow
that earlier, which ends the call as a trap with no span to show. `cpl.js`
then runs the program again with the bytes halved (1.5 MB, 768 kB, ...)
until the interpreter stops it first, and the result is the error above.
The bytes a run was under are its `maxStack`. Output printed before the
limit is kept.

Three failures are not results but exceptions, a `CplError` with a `kind`:
`"stack"` when the browser runs out of stack outside a program's calls
(the parser, on an expression nested tens of thousands deep), `"memory"`
when it runs out of memory, and `"bug"`, with the panic message, for a bug
in the compiler. After any of them `cpl.js` replaces the module with a
fresh instance, so the next call works.

## Tests

```text
$ cargo test -p cpl-wasm            # native: the playground against `cpl`
$ scripts/build-wasm.sh --smoke     # the built module, in Node
```

`tests/parity.rs` runs every program of `crates/cpl-cli/tests/programs`
and every hero program of `web/content/landing/hero` through `cpl` and through this
crate, and compares standard output, standard error, the exit status and
the C++ byte for byte. `tests/smoke.mjs` loads the built `cpl.wasm`
through `cpl.js`, the two files a page loads, and checks what only the
real module can show: the interface, what happens when recursion fills
the browser's stack, and that a call after a trap starts clean.

## Layout

- `src/lib.rs` is `Playground`, the interface in Rust, answering in JSON.
- `src/abi.rs` is the exports of the `.wasm` file.
- `js/cpl.js` wraps those exports; `js/cpl.worker.js` is the worker behind
  `spawn()`; `js/cpl.d.ts` is the types.
- `build.rs` sizes the module's stack for the interpreter.
