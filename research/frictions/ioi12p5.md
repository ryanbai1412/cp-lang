# Friction report: ioi12p5

There is no dream.cp; the ideal is below. Stays are their own `Stay` domain (`opened(t)`, `initial(c) -> ?Stay`). Program 1 replays Belady's rule with one `Set<(Int, Colour)>` and `pop_last`, and program 2 pops spares from a stack.

## Dream

```cp
@grader(two_phase) mod last_supper:
    fn compute_advice(req: [Step]Colour, k: Int) -> [Stay]Bool
    fn assist(advice: [Stay]Bool, requests: Stream<Colour>) -> Stream<Colour>
let next_use: [Step]?Step = ...      // None: never requested again
```

## Two programs run as one stdin program
- **Where:** lines 11-13 and the two `// ----` halves: one file computes the advice, prints it, then runs the assistant on the same in-memory requests.
- **What felt wrong:** the second program must see only the bits and one request at a time. In one file nothing stops the assistant from reading `req` or `next_use`; only the author's discipline separates the halves.
- **Goal it hurts:** hard-to-get-wrong, submission.
- **Ideal code:** the dream's `@grader(two_phase)` module, where `assist` cannot name `compute_advice`'s locals.
- **General fix:** two-phase grader modules with separate scopes (research/frictions/ioi25_migrations.md asks for the same). Owner: core/spec.
- **Severity:** medium.

## "Never again" is the time n
- **Where:** lines 30-31: `var next_use: [Step]Int = n` and `var upcoming: [Colour]Int = n`.
- **What felt wrong:** n stands for "not requested again". It is the right key for the Belady order, but the type does not say so.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `[Step]?Step`, with an optional's order putting `None` after every step (or `?Step` ordered by `desc`).
- **General fix:** an ordering of `?T` keys with `None` last, usable in `Set` tuples. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 433, accepted C++ 1003 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.06 s / 9 MB, accepted C++ 0.12 s / 12 MB, with the same output.
- **Emitted C++:** 19109 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
