# Friction report: ioi25_migrations

Written after the port landed without one (67a6c650c). The findings come from its review (research/reviews/ioi25_migrations.md), its diff against `dream.cp`, and
repros on today's `cpl`.

## A two-phase task's stdin port can't keep its phases apart
- **Where:** `port.cp:133-138`, the driver that runs `send_message` for every site and then
  `longest_path(s)` in the same process, after the `@grader(two_phase)` dream's
  `dream.tests/grader.cpp` (which forks, as the judge does) was kept out of the stdin port.
- **Repro:** add `return (u, v)` as the first line of `longest_path` in `port.cp`, so the second
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

## L0507 asks for a comprehension over a stateful call
- **Where:** `port.cp:135-136`, `var s: List<0..=4> = [0]` /
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

## Already reported elsewhere
- `parents: [n - 1]0..n` and the message list `List<0..=4>` are stored as `ll` in the emitted
  C++ (`Q0<ll> parents`): `read<T>() stores a narrow table wide` in ioi25_festival.md.
- `dream.tests/grader.cpp` instead of a root `grader.cpp`: `A root grader.cpp is linked into
  the stdin port` in ioi25_worldmap.md.

## Measurements

Contestant-written tokens (scripts/ports-page.py tokenizers): port 2329, dream 2243, original
3933 (3872 after `strip_cpp_library`; it includes the two-process grader `main`). On the
`gen.py --max` input (N = 10 000), best of 3 under g++ 14 `-O2`: the port 0.00-0.02 s and
3.2 MB peak RSS, `orig.cpp` 0.00 s and 2.9 MB (1.09x). Both outputs pass `checker.py`.
Emitted C++: 25084 bytes (`cpl build`, judge `generic`).
