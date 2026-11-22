# compiler architecture

The owning lane keeps this current in the same commit as any structural change. It says: the modules and what each is for, the boundaries and invariants, the interfaces other lanes may use (and their consumers), and the known debt. The architecture & tech-debt lane reviews it against the code.

## Modules

To be written by the lane from the code as it stands.

## Interfaces other lanes use

The shared core. The compiler lane owns the code; every consumer lane adds contract tests next to the core for what it needs (for example: the syntax tree round-trips every corpus file, every node has a span, the IR exposes the trace hooks), so a core change cannot land that breaks a consumer. A lane that needs a new interface asks in the compiler worklog instead of working around the gap; architecture & tech-debt reviews every interface change against all its consumers.

| Interface | Consumers |
|---|---|
| Lossless syntax tree and spans | lint-fmt, tools (LSP), diagnostics |
| Typed tree and query API (types, definitions, references, ranges) | tools (LSP, explain, doc), lint-fmt, perf |
| IR | compiler (interpreter), perf (codegen), debugger (trace, stress) |
| Diagnostics and fix API | diagnostics, lint-fmt, tools (LSP, `cpl fix`) |
| Std/builtin boundary | stdlib, tech-debt |

## Invariants

## Known debt
