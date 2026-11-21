# Friction report: agc077a

Dream rewrite toward a synchronized `Block(seen: [Side]Run, real: [Side]Run)`
aggregate instead of four independent segments. The port keeps its current
shape until the gap below closes; the review stays pending a cold rescore.

## Mapping a fixed array loses its index domain

- **Severity:** medium
- **Status:** fixed (Decision 104: `.map` on `[D]T` / `[D, E]T` returns `[D]U` / `[D, E]U`)
- **Goal it hurts:** express
- **Area:** sequences (`plan/STDLIB.md`) + checker
- **Dream lines:** `proto/ports/ecnerwala/dream/agc077a.cp:52`
- **What happened:** `seen.map(f)` on `seen: [Side]Run` returns `List<Run>`,
  so `Block(seen.map(f), real.map(f))` fails with E0302. The domain-indexed
  array has to be rebuilt by hand (`[f(seen[s]) for s in Side]` typed as
  `[Side]Run`), which is the parallel-arrays churn the rewrite removes.
- **General fix:** `.map` on `[D]T` is shape-preserving and returns `[D]U`
  (fixed arrays, nested `[D, E]T`, and slices keep their own rule). Same for
  any element-wise operation that cannot change the length.

## A directed range has to be hand-written as a struct

- **Severity:** medium
- **Status:** open
- **Goal it hurts:** express, coherence
- **Area:** sequences (`plan/STDLIB.md`) + core ranges
- **Dream lines:** `proto/ports/ecnerwala/dream/agc077a.cp:40-45`
- **What happened:** the reconciliation walks one block left to right on one
  side of the big reversal and right to left on the other, so it needs a
  range that remembers its direction. `Range<Int>` has no direction, so the
  dream and port define `struct Run(at, end, d)` with `len`, `span`, `rev`,
  `take` and `drop` by hand, and the output turns it back into `span` for
  printing and for the prefix-sum query (`count_b[s].sum(run.span)`).
- **General fix:** a range value that carries its direction (`(a..=b).rev`
  as a value, or `a.down_to(b)`), with `take(k)`, `drop(k)`, `first`, `rev`
  and `len`, and an unordered `.span` that `Prefix.sum` and printing accept.
  `Run` then disappears and `Block.each` maps a range method over `[Side]`.

## A function parameter applied by `.map` lowers to `std::function`

- **Severity:** low
- **Status:** open
- **Goal it hurts:** performance
- **Area:** codegen
- **Dream lines:** `proto/ports/ecnerwala/dream/agc077a.cp:52`
- **What happened:** `fn each(self, f: Run -> Run) -> Block =
  Block(seen.map(f), real.map(f))` emits `std::function<Run(Run)> t2 = f;`
  inside each `.map` even though `Block_each` is already a template over
  `F0`. With the longer derivation header (embedded in the output) this is
  why the rewrite grew the emitted C++ from 18,866 to 24,529 B; runtime on
  the scoreboard max input stays under the 10 ms floor (2.9 ms vs 2.2 ms).
- **General fix:** when `.map` (or any stdlib higher-order method) receives
  a value already typed by a template parameter, forward it as `F0&&`
  instead of erasing it into `std::function`.
