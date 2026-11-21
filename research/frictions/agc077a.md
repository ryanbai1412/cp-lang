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
