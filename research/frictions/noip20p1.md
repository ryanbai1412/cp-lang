# Friction report: noip20p1

There is no dream.cp; the ideal is below. The port pushes water in `topo(pipes)!` order. Each amount is held as an I128 count of 1 / 60^11 units, so every split by 1 to 5 divides exactly, and the result is reduced by 2, 3 and 5 at output.

## Dream

```cp
var holds = [Junction] u => if u.label <= m: Ratio(1) else: Ratio(0)
for u in topo(pipes)!:
    for v in pipes[u]: holds[v] += holds[u] / pipes[u].len
for u in Junction if pipes[u].is_empty: print(holds[u].num, holds[u].den)
```

## Exact fractions need a hand-picked common unit
- **Where:** lines 14-15 and 23-27: `const SPLITS = 11`, `let unit = I128(60) ** SPLITS`, and the reduction loop over `[I128(2), I128(3), I128(5)]`.
- **What felt wrong:** the program is "add fractions"; the port has to prove that every denominator divides 60^11 < 2^127 and hand-reduce at the end. A change to the statement's limits (more splits, a 7-way pipe) breaks it silently.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** the dream's `Ratio`, reduced on every operation.
- **General fix:** `Ratio` over `I128` (and, for the general case, a big integer) in std, with overflow-checked arithmetic. The C++ uses a hand bignum. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 194, accepted C++ 681 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.06 s / 11 MB, accepted C++ 0.26 s / 17 MB, with the same output.
- **Emitted C++:** 13407 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
