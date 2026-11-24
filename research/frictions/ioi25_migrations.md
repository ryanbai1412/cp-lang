# Friction report: ioi25_migrations

## Dream

```cp
struct Sender:
    n: Int
    plan: List<Phase>
    var tree: JumpTree<Site> = JumpTree<Site>()
    ...
    fn arrive(mut self, at: Site, parent: Site) -> 0..=4:

var sender = Sender(0, [])  // the one value the first run carries from call to call

@grader(two_phase)
fn send_message(n: Int, at: 1..10_000, parent: 0..10_000) -> 0..=4:
    if at == 1: sender = Sender(n, phase_plan(n))
    sender.arrive(Site(at), Site(parent))
```

Both programs keep two candidate lists for a diameter's ends, shrink them over five phases
whose block pair is spelled by one nonzero symbol, and finish with the editorial's three case
tables. The dream is the judge's two `@grader(two_phase)` fns; the port reads the sample
grader's stdin and runs both phases itself, the second seeing only S. The first run's state
is one `Sender` (it was eleven top-level `var`s reset by hand at site 1), every candidate and
end is a `Site`, the endgame lists are `?Site` padded with `None` (it was site 0, a real
site), and each endgame step stores the lists its symbol leaves instead of a `say` that set
a global and returned it.

## A struct field whose type is inferred from its default is emitted as `void`
- **Where:** `dream.cp`/`port.cp`, `Sender`'s `tree`, `u` and `v`.
- **What:** `cpl check` accepts the program, and `cpl build` writes C++ that g++ rejects:
  ```cp
  domain Site = 0..10
  struct S:
      var u = Site(3)
      var t = JumpTree<Site>()
  var s = S()
  print(s.u.ord)
  ```
  gives `error: variable or field 'u' declared void` (and the same for `t`); `cpl run`
  prints `3`. `var n = 0` fields are fine. Both programs annotate the three fields.
- **General fix (codegen):** emit a field's type from the checker's inferred type, as for a
  `let`.
- **Severity:** high (an accepted program that doesn't compile).

## A `@grader` fn can't take a `Site`
- **Where:** `send_message(n: Int, at: 1..10_000, parent: 0..10_000)`.
- **What:** a domain can't cross to the grader (E0501), so the grader fns take plain ranges
  and convert with `Site(at)`; `longest_path` returns `(a!.ord, b!.ord)`.
- **General fix (codegen):** pass a domain whose range is a literal as its `Int` at the grader
  boundary.
- **Severity:** low.

## A grader program carries state from call to call in a top-level `var`
- **Where:** `var sender = Sender(0, [])`.
- **What:** the judge calls `send_message` N - 1 times in one process, so the state must
  outlive a call; the language has no per-run value for a grader, and the placeholder
  `Sender(0, [])` is built only to be replaced at site 1.
- **General fix (codegen, spec):** let `@grader(two_phase)` take a `struct` whose methods are
  the grader fns and whose constructor runs once per phase.
- **Severity:** medium.

## A two-phase task's stdin port can't keep its phases apart
- **Where:** `port.cp`'s driver, that runs `send_message` for every site and then
  `longest_path(s)` in the same process, after the `@grader(two_phase)` dream's
  `dream.tests/grader.cpp` (which forks, as the judge does) was kept out of the stdin port.
- **Repro** (on the port as of eadf73e60): add `return (u, v)` as the first line of `longest_path` in `port.cp`, so the second
  phase reads the first phase's globals instead of decoding `s`. `cpl build` accepts it, and its
  `--max` output passes `checker.py` against `orig.cpp` (exit 0). The judge runs the phases in
  separate processes, so that program scores 0 there.
- **What felt wrong:** the separation the task is about is the one thing the port's gates can't
  see. The port has to be a stdin twin of the dream because `cpl stress` and the `--max` gate run
  stdin programs only, and the twin can cheat by accident (any global the second phase reads).
- **Goal it hurts:** evidence, hard-to-get-wrong, submission.
- **Ideal code:** no stdin twin: the gates run `dream.cp`'s `@grader(two_phase)` fns through the
  package's grader, each phase in its own process.
- **General fix (tools):** let `cpl test`, `cpl stress` and the corpus `--max` gate run a
  `@grader(two_phase)` program as two processes (the first phase's returns piped to the second),
  so the dream is the port. Failing that, run a two-phase stdin port's second phase with the
  first phase's globals reset. This helps every IOI communication (two-run) task; migrations
  is the corpus's first.
- **Owner:** Consumers/Tools.
- **Severity:** high (a wrong-on-the-judge program passes every local gate).
- **Status:** fixed by ba1cf559f: with no `port.cp`, corpus.py's tests and `--max` gates run
  the dream's `@grader(two_phase)` fns through `dream.tests/grader.cpp`, each phase in its own
  process, so `port.cp` is deleted and the dream is the program the gates judge.

## L0507 asks for a comprehension over a stateful call
- **Where:** the deleted `port.cp`'s driver, `var s: List<0..=4> = [0]` /
  `for i in 1..n: s.push(send_message(n, i, parents[i - 1]))`, left as a loop by the review.
- **Repro:**
  ```cp
  var calls = 0
  fn next_id(x: Int) -> Int:
      calls += 1
      calls * x
  var ids: List<Int> = []
  for i in 0..3: ids.push(next_id(i))
  print(ids)
  ```
  `cpl lint` gives `warning[L0507]: a loop that only pushes onto ids is a comprehension` with
  `help: write [next_id(i) for i in 0..3]`.
- **What felt wrong:** a comprehension reads as "what each item is", a pure map; here each item
  is the next step of a state machine, and the loop says that order matters. The port can't
  silence it without `#[allow]` or a rewrite the reviewer called worse (R7).
- **Goal it hurts:** coherence (a lint teaching the wrong idiom), hard-to-get-wrong.
- **Ideal code:** the loop as written, with no warning.
- **General fix (lint):** skip L0507 when the pushed expression calls a fn that assigns a
  top-level `var` or a `var` captured from outside the loop. This helps every interactive or grader port that drives a stateful
  callback per item.
- **Owner:** lint-fmt.
- **Severity:** low.

## Earlier frictions (from the first port)
- **Grader module:** `@grader(two_phase) mod migrations:` builds no function for the grader,
  so the dream is two top-level `@grader(two_phase)` fns.
- **Grader linking:** a `grader.cpp` beside the stdin port was linked into it too, so the
  grader sat in `dream.tests/`. Fixed by 57c83b9a8 (a found `grader.cpp` is linked only into a
  program with a `@grader fn`), and ba1cf559f removed the stdin port itself.
- **Missing names:** `Graph` has no `add_edge`, so the ends are tracked on a `JumpTree`; no
  `cbrt` (`iroot(·, 3)`) or `min_by` (`argmin`).
- **Storage:** `List<0..=4>` and `[n - 1]0..n` are emitted as `vector<ll>` while `cpl explain`
  states the narrow size; owned by Core (memory child), see ioi25_festival.md.

## Measurements

Contestant-written tokens (scripts/ports-page.py tokenizers, as of eadf73e60): dream 2243,
original 3933 (3872 after `strip_cpp_library`; it includes the two-process grader `main`). The
package has no port since ba1cf559f: the gates judge the dream through `dream.tests/grader.cpp`.
On the `gen.py --max` input (a path, the ends moving every step), 7 runs each under g++ 14
`-O2`: the dream under `cpl test` with the grader takes 10-20 ms and 3.17-4.09 MB peak RSS (the
larger of its two phase processes), `orig.cpp` run alone 0.00-0.02 s and 3.97-4.22 MB.
Migrations sorts nothing, so 333c96870 doesn't change it. Before the port was deleted (on
eadf73e60) it took 4.61-4.74 MB against orig.cpp's 3.97-4.10 MB, 1.16x, from the storage
widening above. The dream's S differs from the earlier port's on cases 03 and 14, where padding
with site 0 matched a table entry that `None` now doesn't; both pass `checker.py`.
