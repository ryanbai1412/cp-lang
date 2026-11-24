# Friction Report: noip09p2

## 1. `divisors` by trial division is twice the C++ at the max
- **Where:** `divisors(b1)` (O(sqrt b1)) took 0.38 s vs C++ 0.19 s on 2000 queries at 1.8e9.
- **What felt wrong:** the obvious call is the slow one; `divisors_from_factors(factor(b1))` (Pollard rho) is
  0.05 s and needs a `use`.
- **Goal it hurts:** fast (the obvious way is the fast way).
- **Ideal code:** `divisors(b1)` choosing the factorisation route above 1e6, as `factor` already does.
- **General fix:** `divisors(x)` delegating to `factor` for large `x`.
- **Severity:** medium.

## Measurements
Stress: 30/30 agree with the v0.2 port and AC 5184950 on gen.py inputs; max (2000 queries, b1 = 1837836000) 0.050 s vs C++ 0.133 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
@!multitest
let (a0, a1, b0, b1) = read<(1..=2e9, 1..=2e9, 1..=2e9, 1..=2e9)>()
print(count(gcd(x, a0) == a1 and lcm(x, b0) == b1 for x in divisors(b1)))
```
