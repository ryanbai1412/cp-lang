# Formatter and linter

Status: draft for 0.3.0. Sources: `plan/45-dev-tools.md`, decisions 70,
93, 140, 144, 148.

## `cpl fmt`

One canonical layout, no options: 4-space indents, 100 columns,
`with` lines first in their block, sorted `use` lines, one blank line
between items, trailing commas in multi-line brackets, the compiler
stamp on line 1. Formatting never changes meaning; it is idempotent and
fast enough to run on every save. (Rfmt.1) `cpl fmt` rewrites only the
files and directories it is given (`cpl fmt .` is the whole project); with
no path it formats standard input to standard output, and `--check` with no
path checks the current directory.

## `cpl lint`

Lints flag legal code that is probably wrong, slow or unidiomatic. They
never block a build. Each has a stable code, a default level, a message
that says what to write instead, and an autofix where possible. (Rlint.1) A lint
is declared by the library that owns the names it mentions; the linter
has no hard-coded stdlib names. Levels are set in `cp.toml [lints]` or
in code with `with lint(name: allow)`. (Rlint.2)

The 0.3 set is catalogued in `../diagnostics.md` (codes `L....`),
including: same-place read and write in one statement
(`a[l] += inc(mut a[l])`, `xs[0] = xs.pop()!`); a lazy chain walked
twice or a `.map` recomputed per read; a generator holding a `mut`
lend walked twice; a literal index into a slice (`w[0]`); a domain-
keyed `Map`; `==` on floats; `wide_array`; padding over 20 %; a `var`
never reassigned; an unused result; a `@memo(array)` over the memory
budget; a shadowed top-level `let`.
