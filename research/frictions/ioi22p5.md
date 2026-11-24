# Friction report: ioi22p5

## A `@grader fn` with a local driver builds without `main`

- **Severity:** medium
- **Status:** open (routed to Core)
- **Goal it hurts:** doubt to evidence (the grader-shaped port can't be tested or stressed)
- **Where:** spec.cp's `@grader fn min_cardinality` with the top level playing the sample grader.
- **What happened:** `cpl run` runs the driver, but `cpl build` drops it and emits no `main`, so
  `cpl test` and `cpl stress` stop at the link step. Repro: `@grader` / `fn twice(n: Int) -> Int
  = 2 * n` / `print(twice(read<Int>()))`; `cpl build g.cp -o g.cpp && g++ g.cpp` is
  "undefined reference to `main'".
- **Workaround:** the port leaves `min_cardinality` unmarked.
- **General fix:** keep the top level as the local driver when building for a local run (R10.1.3,
  decision 136); only a judge profile with a harness drops it.

## The machine is simulated in the port instead of supplied by the judge

- **Ideal code (`dream.cp`):** `@grader fn move_inside(i: Int)` etc. as judge-supplied
  declarations, and `pub fn min_cardinality(n: Int) -> Int` alone.
- **What happens today:** an `extern fn` has no local stub that can answer from the hidden
  types (see `research/frictions/ioi18p1.md`), so the port keeps a counting simulator of the
  machine (`held`, `types_with`, `top`) and reads the sample grader's input.
- **General fix:** a way for a local file to supply the judge half of an `extern fn`.

## Measurements

Contestant-written tokens: port 396, original 307 (`corpus/audit.py` count). Emitted C++: 25297 bytes (`cpl build`, judge `generic`).
Max input not timed: the accepted C++ is a grader function with no main.
