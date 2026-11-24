# Toolchain overview

| | |
|---|---|
| Kind | Design |
| Owner | tech-debt |
| Status | Active |
| Describes | `crates/`, `scripts/debt-ratchet.py` (`crate_edges`, `layer_violations`) |

How a `.cp` file becomes a checked program, a run, a trace and one C++17
file for the judge: the pipeline, the crate that owns each stage, and which
crate may depend on which. Each stage's internals are in its own design
doc; `design/compiler.md` covers syntax through codegen.

## The pipeline

```text
         +------------- query database (incremental) --------------+
 text -> | syntax -> names -> types -> IR -> ownership, facts -> opt |
         +---------------------------------------------------------+
              |         |       |          |              |
          fmt/lint     LSP   explain   interpreter     C++ emitter
                                       (traces, wasm)  (judge target)
```

Everything is a query over one database. The CLI, the language server,
the formatter, the linter, stress and the docs generator are clients of
the same queries. None has its own parser or checker, and none re-derives
a fact the checker already computed.

For example, `cpl check a.cp`, a red squiggle in the editor and the
playground's error panel all come from the same call on the same session.
If they ever disagree, that's a bug in the shared code, never a reason to
patch one client.

## Layering

A crate depends only on crates at an earlier stage, plus the report crates.
Tools and editors depend on the front end, never on each other's internals.

- **Allowed:** `cpl-lint` uses `cpl-check`'s facts; `cpl-lsp` calls
  `cpl-lint`'s one entry point.
- **Refused:** `cpl-check` depending on `cpl-ir`, or `cpl-codegen` calling
  into `cpl-interp`. Back ends share the IR, not each other.

The allowed edges are listed in `scripts/debt-ratchet.py`. A new edge fails
the ratchet until it's added to the list in a reviewed commit, and
`layer_violations` counts edges that point the wrong way.

# Reference

## Stages and their crates

The crates, the stage each belongs to and its job are in
[architecture.md](architecture.md#crates); `scripts/debt-ratchet.py` holds the
allowed edges between them.

The stdlib is CP Lang source in `std/`. It gets its powers from general
mechanisms (`design/std-mechanisms.md`), never from the compiler knowing a
std name.

## What every stage preserves

| Concern | Every stage keeps | Checked by |
|---|---|---|
| Meaning | The spec's meaning through every lowering; checks stay explicit until proven | Conformance cases, interpreter-vs-C++ differential runs |
| Spans and names | A source span and name on every IR value, so every event and error maps to source | Trace and diagnostic snapshots |
| Determinism | No hash-order or address dependence | Building twice and comparing bytes |
| One display rule per type | `print`, traces, the debugger and traps show a value the same way | Trace goldens on both engines |
| No privilege | No std name checked by the compiler | `scripts/debt-ratchet.py` (`ir_builtin_names`, `backend_std_names`) |

## Changes

- 2026-11-23: first draft, from Part A of `handbook/design/architecture.md`.
- 2026-11-23: overview first, reference sections last.
