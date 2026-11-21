# 32. Graders, bundling, `@cpp`

**Layer:** core C15: the C++ ABI for `@grader`, the `@cpp` escape hatch, and the bundling contract. Status: [proposed].

## 32.1 `@cpp` overrides

```cp
@cpp("_pdep_u64(x, mask)", includes: "<immintrin.h>", target: "bmi2")
fn pdep(x: U64, mask: U64) -> U64:
    ...                                   // canonical body: interpreter, tracer, debug, fallback
```

The in-language body is the meaning. The C++ string is an optimization: `cp stress` checks it against the
body, and the bundler falls back to the body when the judge profile lacks the target [r7].

## 32.2 Graders

`@grader fn init(…)` and similar export functions with the judge's C signature. Grader arguments never
convert under `io(base:)`.

A grader module's index spaces are sized by the judge, not by the source. A module-level `domain Row` with
no `= range` is fixed by the first grader call that binds an argument of type `[Row]T`: its length becomes
`Row.count`, and every later binding must match (trap otherwise). Module state like `var home: [Col]?Node`
is declared over it as usual; using `Row` before it is bound traps, naming the domain. `@gen` for a grader problem produces the argument tuple. Stress runs the grader
functions through a generated harness.

`@judge_fn(limit: n + 2) fn press(p: Str) -> I32` declares a function the judge provides. Stress
substitutes the `@mock(press)` implementation and counts calls against the limit, trapping at the calling
line in debug, the same as `Wire(limit:)` [p1 F12, r13 P16, p3].

## 32.3 Lowering and bundling

Alts, checkers, generators and interactors are stripped from the submission. The bundler confirms by
re-parsing the emitted C++ that no stripped symbol remains. Debug and release must agree on every stress
input (goal V).

- **Debug and release agree on every input where debug doesn't trap.** Overflow and out-of-range shifts are
  the cases where they can differ [r12 R13].
