# cpl-stress

`cpl stress`: run a solution and a brute on random inputs, stop at the
first case they disagree on, shrink it, and save it as a regression test.

```text
$ cpl stress d.cp --brute brute.cpp
  brute: brute.cpp      checker: none
  FAIL on case 10 (seed 10): WA
  shrunk from 7 numbers to 4:
    3
    0 0 1
  that is: n = 3  a = [0, 0, 1]
  expected:
    1
  got:
    0
  first difference at line 1, token 1: expected `1`, got `0`
  saved d.tests/stress-10.in
```

## Inputs come from the program's own types

There is no generator to write. The types of the top-level `read()` calls
are the input format:

```cp
let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]0..=1e9 = read()
let queries: [0..q](V, V) = read()
```

is two numbers, then `n` numbers, then `q` pairs of vertices. A bound can
use any value read before it, so every generated case is a valid input.
A loop that reads the same typed `let`s on every pass is a row per pass:

```cp
for _ in 0..q:
    let (l, r): (V, V) = read()
```

reads what `let queries: [0..q](V, V) = read()` would. A pass can read a
count and then that many values (`let s: 1..=n = read()`, then
`let plan: [0..s]V = read()`); the count shrinks with its values. A read
stress can't make (in a fn, `read(gen: f)`, a loop that can `break`) is an
error that names it, never left out of the case.

Names that decide how much input there is (`n` and `q` above) are *sizes*.
`--size 8` keeps them at 8 or less so a brute finishes; values stay close
to their lower bound three times in four, so ties and equal elements are
common, and use their whole range otherwise, to reach overflow. The two
ends of every range come up often, because that is where off-by-ones are.

The same seed makes the same case on every machine.

Supported today: ranges, `Int`, `Char`, `Bool`, `Str`, domains, type
aliases, tuples, records, `std.seq.Perm`, `std.graph.Tree`, `std.graph.SimpleGraph`,
`std.graph.ConnectedGraph`, `std.graph.Dag`, `std.graph.Digraph` (with extra edge fields), `[read() for ...]`,
`@!multitest`, and arrays over one or more domains or ranges (`[R, C]T` is
rows of columns). For anything else the error names the type and its line,
and `--gen PROG` takes over: `PROG <seed> <size>` prints one input.

## Shrinking

A failing case is shrunk structurally, never by cutting text, so every
candidate is still a valid input:

1. each size is lowered as far as the failure allows, dropping the array
   elements that no longer exist;
2. single elements are deleted from every position;
3. every value is lowered toward the bottom of its range.

After each change the case is fitted back to the types: `1..=n` values
are clamped into the new `n`. A candidate only counts when it fails the
same way (a wrong answer does not shrink into a crash). Shrinking stops
after 2000 runs.

## Checkers and interactors

Outputs are compared token by token unless the problem has a judge:

```text
cpl stress d.cp --brute brute.cpp --checker checker.cpp
cpl stress d.cp --interactor interactor.cpp
```

A checker is run as `checker <input> <output> <answer>` and accepts by
exiting 0, so a testlib checker works unchanged. An interactor is run as
`interactor <input>` with its stdin and stdout connected to the
solution's, and accepts by exiting 0. Either can judge an output on its
own, so `--brute` is optional with them.

A file that declares `@checker` or `@interactor` is never compared token
by token. Until the compiler can build the co-located judge, `cpl stress`
stops and asks for it built, instead of calling right answers wrong.

## Programs

A program is a `.cp` file (built through the compiler's C++ back end, the
one behind `cpl build`), a `.cpp` file, a `.py` file, or an executable.
C++ is compiled once with `g++ -std=c++17 -O2` and cached by content
under `CPL_CACHE`, default the system temp directory.

The solution is the file itself; `--fast PROG` runs something else in its
place. The brute, generator, checker and interactor can each be any kind
of program, so a brute can be a second `.cp` file.

## For editors

```text
$ cpl stress d.cp --brute brute.cpp --format json
{"status":"fail","cases":10,"counterexample":{"case":10,"seed":10,
 "input":"n = 3  a = [0, 0, 1]","raw_input":"3\n0 0 1\n","expected":"1\n",
 "got":"0\n","verdict":"WA","stages":[],"saved":"d.tests/stress-10.in"}}
```

(one line in practice). `stages` is reserved for `spec` stage diffs and is
empty for now. The exit code is 0 when every case agreed, 1 on a failure
or when the run could not start, and 2 for a usage error.

## Layout

- `spec.rs`: the input format, from the `read()` types.
- `gen.rs`: cases, rendering, and the seeded generator.
- `shrink.rs`: structural shrinking.
- `program.rs`: building and running programs, with time and peak memory.
- `judge.rs`: token comparison, checkers, interactors.
- `run.rs`: the stress loop and saving regressions.
- `cli.rs`: `cpl stress`.

```text
cargo test -p cpl-stress
```
