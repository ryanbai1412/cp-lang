# Measuring and improving performance

| | |
|---|---|
| Kind | Guidelines |
| Owner | perf |
| Describes | `crates/cpl-codegen/`, `bench/`, `cpl bench`, tuned variants (`perf.cp`), `plan/domains/perf/metrics.toml` |

Read this before you claim a program is fast, compare CP Lang with C++, add
an optimization to codegen or write a tuned variant.

## A good comparison, by example

`corpus/problems/cf2084h/` has the contestant's C++ (`orig.cpp`), the
idiomatic port (`port.cp`) and a tuned port (`perf.cp`). All three use the
same algorithm: a stack of run lengths, O(n). The harness builds each one
with the judge profile's compiler and flags, runs it on the same generated
max-size input (`gen.py`), takes the median of 5 runs of at least 10 ms,
and checks that all three print the same output. The result line reads
`cf2084h 1.23x idiomatic, 0.95x tuned`.

A bad comparison: timing a port that uses a segment tree against C++ that
uses a Fenwick tree, or against C++ built with `-O0`, or on a 10-line input
where process start-up dominates. Each of these measures something other
than the language.

## The rules

1. **Compare matched algorithms and layouts.** The C++ reference and the
   port use the same algorithm, the same complexity and the same data
   layout (array of structs against array of structs). If they differ, fix
   the reference or the port first. A pair that can't be matched goes in
   `bench/unmatched.txt` with its reason; it is never silently dropped.
2. **Measure the submission.** Time the C++ that `cpl build` emits, built
   with the judge profile's compiler and flags, never the interpreter and
   never a debug build. Use inputs at the problem's stated maximum, plus a
   degenerate case (all equal, sorted, a path graph) when the algorithm has
   one.
3. **Check the answer before the time.** A run counts only if the port and
   the reference print the same output on the timed input.
4. **Account for every pair.** Each run reports how many pairs built, ran,
   were skipped and why. A pair that fails to build is a bug to fix or a
   row in `bench/unmatched.txt`, not a gap in the table.
5. **Meet the bars.** Over the perf set, as `bench/v03_perf_bench.py`
   reports them:

   | Measure | Bar |
   |---|---|
   | Idiomatic runtime, p50 | at most 1.1x matched C++ |
   | Idiomatic runtime, p95 | at most 1.2x |
   | Idiomatic runtime, max | at most 1.5x |
   | Tuned runtime, max (`runtime_tuned_max`) | at most 1.1x |
   | Zero-cost pairs (`bench/zero-cost/`) | at most 1.05x against hand-written C++ and against the direct `.cp` |
   | Emitted C++ size | under 32 KB; 64 KB is the judges' hard limit |

   Current numbers live in `plan/domains/perf/metrics.toml`, not here.
6. **Every outlier gets a tuned variant or a logged gap.** For each program
   over 1.1x, keep the idiomatic port and add `perf.cp` beside it. Tune only
   with the language: narrower widths and range types, `unchecked:`,
   layout, reserving, a different std facility. Never inline C++ to pass
   the bar. If the tuned variant still can't reach 1.1x, log the reason as
   a language or codegen gap in `plan/domains/perf/PLAN.md`, with the owner
   who can close it.
7. **Behaviour never changes.** An optimization may not change what a
   program prints, its output order, how many times a call with effects
   runs, whether or where it traps, or whether a lazy view is evaluated
   again on each read. Gate each rewrite on a fact the compiler has proven:
   `cpl_ir::effects` says the call is pure, or the Uses table says the
   difference can't be observed (the value isn't mutated, moved or
   escaped). "Probably pure" is not a gate.
8. **Prove it in both engines.** Before landing an optimization, run the
   differential and parity tests (`cargo nextest run -p cpl-cli --test
   differential`) and `cpl-conform diff`. Add the program that motivated it
   as a parity case under `crates/cpl-cli/tests/parity/`, with an effect
   counter when evaluation count matters.
9. **Ratchets only go down.** The counts of deep copies and of
   `std::function` in emitted C++ (`crates/cpl-cli/tests/read_only_copies.rs`)
   and the size limits are shrink-only. If your change raises one, fix the
   change; never raise the bound. When your change lowers one, lower the
   bound in the same commit.
10. **Fix the class, not the case.** A copy, an indirect call or a lost
    check elimination in one program is a symptom. Find the rule codegen is
    missing and fix it for every program, then add the program as a
    regression.
11. **Make every implicit choice visible.** If codegen copies a value,
    calls through `std::function`, fuses, sinks or drops a check, `cpl
    explain` (text and `--json`) says so, with the source position and the
    reason.
12. **Make it reproducible.** Record the command, the commit, the judge
    profile, the input generator and its seed, and the machine in
    `plan/domains/perf/worklog.md` with each number you publish. Time on a
    quiet machine, one job at a time (`--jobs 1`).
13. **Never submit to a real judge** to measure. Use the local profile that
    mimics it.

## Checklist

- [ ] The reference uses the same algorithm and layout as the port.
- [ ] The port and the reference agree on the timed input.
- [ ] The input is at the stated maximum; degenerate cases are included.
- [ ] Every pair is built, timed or listed with its reason.
- [ ] Each outlier over 1.1x has `perf.cp` or a logged gap with an owner.
- [ ] The rewrite is gated on proven purity or unobservability.
- [ ] Differential, parity and conformance pass; the motivating program is a
      parity case.
- [ ] No ratchet went up; any that went down are lowered in the commit.
- [ ] `cpl explain` names any new implicit choice.
- [ ] The numbers, command and seed are recorded in the worklog.
