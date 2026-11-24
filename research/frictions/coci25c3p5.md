# Friction report: coci25c3p5

Rethought lightly. The v0.2 port read n, a and b as untyped `Int`s and summed adjacent pairs with `i.prev!` behind an `i != Fan.first` guard. The v0.3 port types all three from the statement and takes each pair with `for r in Fan if let l = r.prev`, so there is no unwrap.

No language gap.

## Measurements

gen.py --max: port 0.08 s, accepted C++ 0.04 s, same output.

## Port 2x the C++ at --max

- **Where:** the whole port: `print(n + 1 + sum(b[i] + 1 for i in Fan) - sum(shared(l, r) for r in Fan if let l = r.prev))` and the `gcd`/`lcm` in `shared`.
- **What felt wrong:** 0.08 s against 0.04 s (gen.py --max) is over idiomatic-code/R10's 1.1x; the gap is not yet attributed to one operation.
- **Goal it hurts:** the obvious way is the fast way
- **General fix:** profile the `--max` case against the C++ and route the hot operation (generator `sum`, `gcd`/`lcm`) to its owner.
- **Severity:** low (far under the limit). **Owner:** perf.
