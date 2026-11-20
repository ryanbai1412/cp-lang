# 5. Debugging: traces, snapshots, stress testing

Status: decided unless a section says *proposed*. Folded from `debugging.md`
(now in `archive/`); this copy is the source of truth.


A walkthrough of one problem from first save to accepted, showing exactly
what the tools print at each step. Everything here is a target design (goal
G5); see `plan/40-tooling.md` §6–7 for the mechanisms.

**The problem.** Array `a` of length n. Each query `(l, r)` asks for the
number of distinct values in `a[l..=r]`. Offline solution in three stages:
compress values, compute `prev[i]` (the last earlier position with the same
value), then answer queries sorted by `r` with a Fenwick tree counting
positions whose `prev < l`.

```
let n: 1..=2e5 = read()
domain I = 1..=n
let a: [I]0..=1e9 = read()
let q: 1..=2e5 = read()
domain Q = 1..=q
let qs: [Q](l: I, r: I) = read() where l <= r

fn last_seen(a: [I]Int) -> [I]?I:               // stage 1
    var seen: Map<Int, I> = {}
    [I] i => { let p = seen.get(a[i]); seen[a[i]] = i; p }

fn answer(prev: [I]?I) -> [Q]Int:               // stage 2
    var fw = Fenwick<I, Int>(Sum<Int>)
    var ans: [Q]Int = 0
    var r = I.first
    for k in sort_by(Q, k => qs[k].r):
        ...
    ans

let prev = last_seen(a)
print(answer(prev), sep: "\n")

@brute(answer)
fn answer_slow(prev: [I]?I) -> [Q]Int =
    [Q] k => Set(a[i] for i in qs[k].l..=qs[k].r).len
```

## 1. First save: compile errors

```
$ cp test
error[E0102]: `k` is a Q, but `prev` is indexed by I
  --> d.cp:19:20
   |
19 |         if prev[k] ?? I.first < qs[k].l: ...
   |                 ^ expected I, found Q
   |
   = note: Q = 1..=q (queries), I = 1..=n (positions)
help: `r` is the I being swept here:
   |
19 |         if prev[r] ?? I.first < qs[k].l: ...

error[E0311]: comparing `?I` with `I` needs a choice for none
  --> d.cp:19:12
   |
19 |         if prev[r] ?? I.first < qs[k].l
   |            ^^^^^^^^^^^^^^^^^^ `none` means "first occurrence";
   |                               should it count as before every l?
help: `none` sorts before every value of I, so compare directly:
   |
19 |         if prev[r] < qs[k].l
```

- **One root cause, one error;** nothing cascades from the first mistake.
- **Explained in the problem's terms:** "queries" and "positions" come
  from the domain declarations and their comments.
- **The help is code you can paste,** with the changed part underlined.
  `cp fix` applies a suggestion when there's exactly one.

## 2. Samples: a WA, and the stages that produced it

```
$ cp test
  1  ok   2 ms
  2  WA   2 ms
     expected  got
     3         3
     2         3   <- line 2
     1         1
  1/2 passed (debug)   cp run d 2.in --trace to see stages
$ cp run d 2.in --trace
  n = 5   a: [I] = [1, 2, 1, 3, 2]
  q = 3   qs: [Q] = [(l: 1, r: 3), (l: 2, r: 5), (l: 4, r: 4)]
  last_seen(a) -> [I] [none, none, 1, none, 2]           [s1]
  answer(prev) -> [Q] [3, 3, 1]                          [s2]
```

- **No prints were added.** Every top-level `let` and stage result is
  shown, pretty-printed by type: domain-labelled arrays, records with field
  names, `none` rather than a sentinel like `-1` or `0`.
- `last_seen` is right (`prev[3] = 1`, `prev[5] = 2`), so the bug is in
  `answer`, visible without adding a single print.

## 3. Stress: a minimal counterexample, side by side

```
$ cp stress
  brute: answer_slow     inputs: from read() types, small sizes first
  312 cases (8 threads) ... FAIL at case 313, shrunk in 0.1 s to:
    n = 2  a = [7, 7]  q = 1  qs = [(l: 2, r: 2)]
  stage          fast            brute
  last_seen      [none, 1]       (same fn)
  answer         [0]             [1]          <- first difference
  saved i6 (scratch) and snapshot answer-1
```

- **Inputs come from the program's own `read()` types,** so there's no
  generator to write, and constraints like `l <= r` are respected.
- **Shrinking** reduces n, values and query count while the failure
  persists, so the case is small enough to reason about.
- **The first stage that differs is highlighted.** With `@brute` on more
  than one fn, this pinpoints the broken part, not just the wrong output.

## 4. Snapshot: iterate on one stage

```
$ cat .cp/snaps/answer-1.snap
prev: [I] = [none, 1]
reads: n = 2, a = [7, 7], q = 1, qs = [(l: 2, r: 2)]
$ cp run d --stage answer-1 --trace answer
  answer(prev: [none, 1])
    r = 1: fw.add(1, 1)              fw = [1, 0]
    r = 2: prev[2] = 1 < l = 2       fw.add(2, 1); fw.add(1, -1)
    k = 1: fw.fold(2..=2) -> 0       <- expected 1
  -> [0]
```

- The snapshot is readable text, so you can edit it (e.g. change `a`) and
  rerun.
- `--stage` skips reading input and earlier stages, so the loop is
  edit → rerun → look, in milliseconds.
- `--trace answer` steps through the fn's loop, showing the variables it
  assigns. Here the trace shows the add happens at the wrong index. Fix,
  rerun the stage, then `cp snap keep answer-1` makes it a permanent
  regression case for `cp fntest`.

## 5. Runtime errors never segfault

```
$ cp run d i3
error: index out of range
  --> d.cp:24:22
24 |         ans[k] = fw.fold(qs[k].l..=r)
   |                          ^^^^^^^^^^^ empty range: l = 4 > r = 3
   = call stack: answer (d.cp:24) ← top level (d.cp:31)
   = input: i3, line 7 (query 2)
   = snapshot: answer-2 saved; rerun with cp run d --stage answer-2
```

- Overflow, `none` unwraps, failed `assert`s and recursion depth report the
  same way, with the input position.
- A crash saves a snapshot of the stage it happened in.
- Release builds keep a source map: a crash prints the source line, without
  values.

## 6. Too slow or too big

```
$ cp run d max.in --release
  output: 200000 lines   time 0.31 s / 1 s   memory 9.8 MB / 256 MB
$ cp explain d
  answer: O(q log n + n log n)   ~ 7.4e6 ops at n = q = 2e5
  last_seen: Map<Int, I> ~ 2e5 inserts  (hint: HashMap<Int, I> or
             compress(a) gives a domain and an array instead of a map)
  storage: a [I]0..=1e9 4 B x 2e5 = 0.8 MB, prev [I]?I 4 B x 2e5 = 0.8 MB
warning: line 12 loops over Q inside a loop over I: up to 4e10 iterations
```

- Complexity comes from the declared ranges; warnings point at the loop.
- `cp explain` shows bytes per element and the representation chosen.

## 7. Submit

```
$ cp submit
  d.cpp  11.8 KB / 64 KB   compiled -O2 ok   samples ok   copied
```

## 8. Interactive and multitest

- **Multitest:** traces, snapshots and errors are grouped by test number
  (`test 3 of 7`), and stress shrinks the number of tests too.
- **Interactive:** `cp run` links to the local `@interactor`, and the trace
  shows the exchange:
  ```
  > ? 3        < 1
  > ? 5        < 0
  > ! 4        judge: correct (2 queries, limit 20)
  ```
  On failure, the interactor's hidden state is printed.

## 8b. Filtering the trace

### Quiet defaults (hard rules)

- About 20 lines on a typical run: top-level stages only. No nested calls,
  loop steps or helpers unless asked.
- Each value fits on one line, truncated to the terminal width with a count
  of what's hidden.
- Loops fold to the first 3 iterations plus the last (`… 1,997 more`).
- Small helpers are never shown: one-liners, `@get`s, std internals.
- Past ~200 lines the terminal trace stops and points to the HTML trace.

### Filters

- **Only some fns** — `--trace answer,last_seen`, `--trace 'seg*'`, `--trace -helper`
- **Nested calls** — `--depth 2`, `--trace all`
- **One multitest case** — `--test 3`
- **Only some calls or iterations** — `--when 'k == 5'`
- **Every write to one place** — `--watch 'dist[7]'` (line, old → new)
- **What happened right before a crash** — `--last 50` (ring buffer)
- **Only what changed per loop step** — `--changed`
- **Value width** — `--width 20`
- **Fast vs brute** — `--diff brute`, from the first difference

In source, for traces you want every time: `@trace`, `@trace(k == 5)`,
`@notrace` on a noisy helper. Stripped from submissions.

### The HTML trace is the main view

Debug runs always record the full trace. The terminal shows only the
summary above plus one line, `trace: .cp/trace/d-i1.html`, which opens in
the browser. The file is self-contained (no server), so it can be shared.

It reads as a guided walkthrough of the whole execution, not a log:

- **Stages are chapters:** input, each top-level `let` and stage, then
  output. Each chapter opens with its inputs and result, rendered by type.
- **A timeline scrubber** steps through events inside a stage (loop
  iterations, calls, writes). Data views (arrays, grids, Fenwick/segtree
  contents, maps) update as you step, and the cell just written is
  highlighted.
- **"Jump to first wrong value"** when a brute or expected output exists:
  the walkthrough opens at the earliest event where fast and brute diverge.
- The source line for the current event is shown next to the data.

- A collapsible call tree; loops and nested calls start folded.
- Search for any value and a `when:` filter box, so the filter flags above
  become UI and stay optional in the terminal.
- Values rendered by type: grids as tables, domain-indexed arrays with
  their labels, sets and records formatted, long arrays scrollable.
- Fast vs brute side by side, with a jump to the first difference.
- Each event linked to its source line, shown inline.
- A "snapshot this call" button that copies the `cp run d --stage` command.

## 9. Principles

1. **Speak in source terms.** Never show C++, raw indices without their
   domain, or sentinels. Show values with their types.
2. **One problem, one message,** with the fix as code.
3. **No prints to add or remove.** Tracing comes from the program
   structure: stages, `let`s, types.
4. **Every failure becomes a small, replayable artifact:** a shrunk input
   (`iN`), a snapshot of one stage, a regression case.
5. **Fast loops:** samples in debug well under 200 ms, a stage rerun in
   milliseconds.
6. **Quiet on success, detailed on failure.**

## 10. What it takes to build

- The debug interpreter over typed IR (values with their types at runtime).
- Source spans on every IR node, and a source map in generated C++.
- Derived `Show` for every type, including domains, optionals and grids.
- I/O recording per part, so any stage can be snapshotted and replayed.
- A shrinker over the `read()` spec.
- A diagnostics framework: error codes, spans, suggestions, `cp fix`.
