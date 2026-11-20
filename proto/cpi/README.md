# cpi: the reference interpreter

A tree-walking interpreter for the library subset of CP Lang: modules and
`use`, `pub`, generic structs and functions, traits and `impl`, operation
values and their laws, struct-owned domains, `yields` generators and
`@test`. It exists so `stdlib/` and local libraries can be written and
tested before the Rust compiler accepts that syntax, and afterwards as the
slow, obviously correct side of differential tests.

```text
$ python3 proto/cp.py test stdlib examples/local-lib   # run every @test
$ python3 proto/cp.py test stdlib -k lca               # only matching names
$ python3 proto/cp.py doc stdlib                       # Markdown from /// comments
$ python3 proto/cp.py lib-run solution.cp < input      # run a solution that imports libraries
```

It reuses the lexer and the statement and expression parser of
`proto/cpc/` and adds the declarations that compiler does not have
(`parser.py`). `interp.py` evaluates; `values.py` holds the run-time
values; `builtins.py` is the prelude; `runner.py` finds and runs tests
and prints the docs.

## What it checks

Everything is checked at run time, and a failure names the source line.

- An index must belong to the array's domain: `a[e]` with an edge index
  into a `[V]T` fails with "this array is indexed by V, got a E".
- Domain values stay in range; `Int` arithmetic that leaves 64 bits traps.
- `/` and `%` are Euclidean; dividing by zero stops the program.
- `none` cannot take part in arithmetic or ordering.
- `laws_hold(op, samples)` tries the laws an operation value declares on
  every combination of the samples.
- Outside a struct's body, `Name(...)` calls its `fn new` when it has
  one, so a structure with invariants cannot be built around them.

`python3 proto/cpi/checks_test.py` runs one failing program per check and
compares the message.

## What it does not do

It does no static type checking, so a type error on a path no test takes
goes unnoticed, and annotations are trusted rather than verified. It does
not enforce the borrow rules. It is slow: fine for tests on small random
inputs, not for judging.

## Resolving imports

`use std.x` is `stdlib/x.cp` in this repository. Any other root is a
library name: the interpreter walks up from the importing file looking
for a `lib.toml` whose `name` matches, or a `cp.toml` that maps the name
to a path under `[libs]`.
