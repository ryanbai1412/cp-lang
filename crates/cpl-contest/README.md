# cpl-contest

The workflow around one problem file: `cpl new`, `cpl test`, `cpl bench`
and `cpl watch`. A problem is `d.cp` plus an optional `d.tests/` folder;
every command takes `d` or `d.cp`.

## `cpl new`: a problem from its constraints

```text
$ cpl new d --constraints "n, q: 1..=2e5; a: [n]0..=1e9; queries: [q](1..=n, 1..=n)"
  created d.cp
  created d.tests/ (no samples yet)
```

writes the input already read into typed values:

```cp
let (n, q): (1..=2e5, 1..=2e5) = read()
let a: [0..n]0..=1e9 = read()
let queries: [0..q](1..=n, 1..=n) = read()
```

Constraints are `name: type` separated by `;`. Several names share a type
(`n, q: 1..=2e5`), and `[n]T` is short for `n` values, `[0..n]T`. The
types are checked before anything is written: they are what `cpl stress`
and `cpl bench` make inputs from, so a scaffold that would not stress is
refused with the reason.

`--companion problem.json` (or `-` for stdin) reads a
[Competitive Companion](https://github.com/jmerle/competitive-companion)
problem: its title and URL become comments, its limits become the
`// limits: 1000 ms, 256 MB` line, and its samples become `d.tests/1.in`,
`1.out`, and so on. With no name given, the file is named after the
problem (`D. Range Max` is `d.cp`).

If the nearest `cp.toml` has a `template.cp` next to it, the template is
put after the reads. An existing file is never overwritten.

## `cpl test`: every case in `d.tests/`

```text
$ cpl test d
  1             ok      3 ms
  2             WA      2 ms
     expected: 7
     got:      6
     first difference at line 1, token 1: expected `7`, got `6`
  stress-48213  ok      2 ms
  2/3 passed
```

A case is `N.in` with an optional `N.out`. Numbered samples run first, in
order; the rest (your own cases, the ones `cpl stress` saved) follow by
name. A case with no `.out` is run and its output shown. Each run gets
the problem's time limit.

Problems with many right answers, and interactive ones, are judged the
way a judge does it:

```text
cpl test d --checker checker.cpp        # checker <input> <output> <answer>
cpl test d --interactor interactor.cpp  # interactor <input>, piped to the solution
```

Both accept by exiting 0. A file that declares `@check` or
`@interactor` is never compared token by token: until the compiler can
build the co-located judge, `cpl test` asks for it built.

## `cpl bench`: the largest inputs the problem allows

Samples are small; a judge's tests are not. The input types say how large
an input can be, so the worst cases are made without writing a generator:

```text
$ cpl bench d
  random           600002 numbers  TLE   2002 ms      5 MB
  all smallest     600002 numbers  ok      28 ms      5 MB
  all largest      600002 numbers  ok      80 ms      5 MB
  ascending        600002 numbers  ok      42 ms      6 MB
  descending       600002 numbers  ok      35 ms      5 MB
  alternating      600002 numbers  TLE   2002 ms      5 MB
  path             600002 numbers  ok      33 ms      5 MB
  star             600002 numbers  TLE   2002 ms      5 MB
  slowest: random at 2002 ms, 200% of the 1000 ms limit
  memory: 6 MB of 256 MB
  3 shapes are over the limits
```

Every size (`n`, `q`) is at its maximum. The shapes are the ones that
break solutions which pass the samples: all-equal values (quadratic
quicksort, hash collisions on one key), sorted and reversed input,
alternating extremes (overflow), and, when the input has a list of pairs,
the edges as one long path (deep recursion) and as a star (one huge
adjacency list). A run is stopped at twice the time limit, so the report
says how far over a slow solution is.

Memory is the peak resident size, sampled while the program runs (Linux).
The exit code is 0 only when every shape is within both limits, and
`--format json` prints the same numbers for a script.

Limits come from `--time-limit` and `--memory-limit`, else the problem's
`// limits:` line, else `[limits]` in the nearest `cp.toml`:

```toml
[limits]
time_ms = 2000
memory_mb = 256
```

else 2000 ms and 256 MB.

## `cpl watch`: on every save

```text
$ cpl watch d
  check: no errors
  1  ok      1 ms
  2  ok      1 ms
  2/2 passed

--- d.cp changed
error[E0201]: no `m` in scope
  --> d.cp:2:7
  |
2 | print(m)
  |       ^
  = help: declare it first: `let m = ...`

d.cp: 1 error
  check: 1 error to fix
```

A round reports errors, then lints, then runs the cases; errors stop the
round before the tests. It runs again when the source, anything in
`d.tests/`, or the `--fast` program changes. The checker's database lives
across rounds, so a save only re-checks what changed. `--once` runs one
round and exits with its result, for scripts.

## Which program runs

The commands build and run the problem itself: `d.cp` goes through the
compiler's C++ back end, the one behind `cpl build`, and the C++ is
compiled with `g++ -std=c++17 -O2` and cached. `--fast PROG` runs
something else in its place: a `.cpp` file, a `.py` file, or an
executable. `cpl watch` says the tests were skipped when the solution
cannot be run.

## Layout

- `new.rs`: constraints, Competitive Companion, the scaffold.
- `problem.rs`: limits and the cases in `d.tests/`.
- `test.rs`: running and reporting cases.
- `bench.rs`: worst-case shapes and the report.
- `watch.rs`: change detection and the check, lint, test round.
- `json.rs`: a small JSON reader.

```text
cargo test -p cpl-contest
```
