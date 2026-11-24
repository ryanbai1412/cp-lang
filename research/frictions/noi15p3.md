# Friction Report: noi15p3

## 1. A runtime `Mod<p>` alias in a fn signature does not check
- **Where:** `fn join_x<T>(f: [Mask, Mask]T, s: Mask) -> [Mask, Mask]T`.
- **What felt wrong:** with `type Z = Mod<p>` and `p` read at run time, `fn join_x(f: [Mask,
  Mask]Z, ...)` fails with E0005 "expected a compile-time constant integer", so the helpers are
  made generic over the element type.
- **Goal it hurts:** writable from the docs; the obvious signature should work.
- **Ideal code:** `fn join_x(f: [Mask, Mask]Z, s: Mask) -> [Mask, Mask]Z`.
- **General fix:** runtime-modulus aliases in fn signatures (sent to Core).
- **Severity:** low.

## 2. Rethought
The v0.2 port hand-rolled a trial-division sieve, `mod_pow` and a 3^8 ternary mask with
inclusion-exclusion. The re-port is the two-subset DP over `Subset<Small>` pairs with
`Sieve.factor` and `Mod<p>`.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 500) 0.47 s vs C++ 0.16 s, same
output.
