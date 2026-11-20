# cpl-verify

The verification harness: a differential fuzzer, a `sizeof` audit, the two
line counts and a latency benchmark. It tests the Python prototype in
`proto/cpc/` today and takes `cpl-interp` and `cpl-codegen` through the
same two traits when they can run programs.

```text
$ cargo test -p cpl-verify                 # all of it, under a minute
$ cargo run -p cpl-verify -- fuzz --seeds 0..2000
$ cargo run -p cpl-verify -- layout
$ cargo run -p cpl-verify -- loc
$ cargo run -p cpl-verify -- bench
```

It needs `g++` and `python3` on the path and has no crate dependencies.

## The fuzzer

`fuzz::gen` writes a random program from a seed, `fuzz::eval` is a
reference evaluator that says what the program must print, and `diff`
sends the program down every path of the toolchain and compares. A path
passes when it exits 0, prints exactly the expected text, and neither
AddressSanitizer nor UndefinedBehaviorSanitizer reports anything. The
C++ is always built with `g++ -std=c++17 -O2 -fsanitize=address,undefined`.

```text
$ cargo run -p cpl-verify -- gen 5 --expect
fn f1(p2: Int, p3: Bool) -> Int:
    if max(p2, 13) > (8 * p2):
        return p2 % (-9)
    for i4 in 1..3:
        var b5: Bool = p3
    ...
--- expected output
5 NO 8 0 1 -85 4
4 4
```

When a seed fails, the program is cut down to a few lines that still fail
the same way, and both versions are written to `target/cpl-verify/failures`:

```text
seed 146: WrongOutput in proto/cpc: line 1: expected `NO 52 720907`, got `NO 46 720907`
var x12: Int = -41
var xs14: List<Int> = [x12 / 2]
x12 += 0
print(xs14)
```

The reference evaluator follows `plan/`: `Int` is 64-bit and overflow is
an error, `%` is never negative and `/` is the quotient that goes with it,
`and` and `or` short-circuit, indexing is checked. The generator stays
inside what the plan pins down, so a disagreement is a bug and not an
open question: names are never shadowed, fns are pure, a divisor is
nonzero by construction, and a list is not changed inside a loop that
reads it.

For each runtime error the toolchain promises to catch, every seed also
gets a variant that must stop with a nonzero exit code and no sanitizer
report. The prototype promises only `index` today; pass
`--traps overflow,div-by-zero,index` for a toolchain that traps all three.

### Plugging in the Rust compiler

`backend::Transpiler` is ".cp to C++17" and `backend::Interpreter` is
"run .cp". Until the crates export an API, the `cpl` binary plugs in by
command line, with `{src}` and `{cpp}` filled in (the flags below stand
for whatever `cpl` ends up taking):

```text
$ cargo run -p cpl-verify -- fuzz \
    --transpile 'target/debug/cpl build {src} --emit-cpp {cpp}' \
    --interp 'target/debug/cpl run {src}' \
    --traps overflow,div-by-zero,index
```

With `--interp`, every program goes through the interpreter as well as
through the sanitized C++, and both must match the reference. Once
`cpl-interp` and `cpl-codegen` have a library entry point, implement the
two traits for them in `backend.rs` and make them the default.

## The `sizeof` audit

`layout/core.cp` declares one variable per core type, with the size
`plan/80-validation.md` 8.2 promises written beside it:

```text
var parent: [V]?V = none                    //! sizeof parent[] == 4
```

`cpl-verify layout` transpiles the file, asks the C++ compiler for the
real sizes and fails on any difference. A size a backend is known to miss
goes in `layout/core.gaps` (`parent[] 8`); the audit then fails when that
number moves in either direction, including when the gap closes.

## Line counts

`cpl-verify loc` reports both numbers from `plan/80-validation.md` 8.1 for
every port in `proto/ports`: lines against the accepted submission, and
lines against the submission with library code taken out. Blank lines and
comments never count. What counts as library code is written down per
port in `<name>.strip`, as line ranges of `<name>_orig.cpp`:

```text
1-30   template header
35-56  DSU
58-82  Dijkstra
```

A port without a `.strip` file is reported as not reviewed; it is never
counted as if nothing were library code.

## The latency benchmark

`cpl-verify bench` times the stages a contestant waits on, for each
program with an input in `bench/`, and prints the median against the
budget: check 50 ms, debug run 200 ms, release run with a precompiled
header 500 ms, LSP diagnostics and hover 100 ms, emitted C++ 64 KB.
`--enforce` makes an over-budget stage exit 1. A stage whose tool does
not exist yet prints `not measured`; pass `--interp` and `--lsp 'cpl lsp'`
when they do.

## What the prototype does not do yet

The harness is configured around these, and each is a finding to settle:

- Overflow and division by zero are undefined behavior in the emitted
  C++ (UBSan reports them) where the plan says they stop the program.
- Pushing to a list inside a `for v in xs` over it is a use-after-free;
  the borrow rules that forbid it are not in the prototype.
- `for i in 0..xs.len` reads `xs.len` again on every iteration. The plan
  does not say whether a range's bounds are evaluated once.
- `plan/11-values-and-types.md` says `/` rounds toward negative infinity
  and `%` is never negative. Both cannot hold for a negative divisor
  (`7 / -2`): the prototype and the reference evaluator keep `%`
  non-negative, giving `-3` remainder `1`.
