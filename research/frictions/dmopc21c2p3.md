# Friction report: dmopc21c2p3

## `@!multitest` drops the bound on the count

- **Where:** `@!multitest` (N <= 1000 hidden numbers)
- **What felt wrong:** the lint asks for `@!multitest` in place of `let n: 1..=1000 = read()` and the loop, but the divider takes no bound, so the statement's N <= 1000 is no longer in the program.
- **Goal it hurts:** types carry the proofs
- **Ideal code:** `@!multitest(1..=1000)`
- **Severity:** low

The old port judged itself with an inline grader over the hidden numbers; it now talks to the judge through `Wire` and `interactor.py` judges it.

## Measurements

gen.py --max (1000 numbers of 60 bits): port 0.60 s, accepted C++ 0.63 s, both under 32000 guesses (with the Python judge).

## Dream

```
let wire = Wire()
@!multitest(1..=1000)
let depth: 0..=60 = wire.ask("0")
```
