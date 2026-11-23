# Friction: `sac22cc2p5`

## A `[Bool]` comprehension binds its index as an `Int`

- **Where:** `spec.cp:7-8`, `fn half_of(p: Bool) -> Fenwick<I, Int> = ...` and `var half: [Bool]Fenwick<I, Int> = [half_of(false), half_of(true)]`.
- **What felt wrong:** the table is keyed by `Bool` and indexed `half[odd(i)]`, but `[Bool] p => ... odd(i) == p ...` is E0302 "can't compare a `Bool` with a `Int`", so the two entries are built by a named fn called twice, in the literal's `false, true` order.
- **Goal it hurts:** coherence, express
- **Ideal code:** `var half = [Bool] p => Fenwick([I] i => if odd(i) == p: a[i] else: 0)`
- **General fix:** a comprehension over a key type binds its parameter at that key type (`[Bool] p` gives `p: Bool`, as `[Dir] d` should give `d: Dir`). Repro: `let x = [Bool] p => if p: 1 else: 0` is E0302 "expected `Bool`, found `Int`". Same gap as `research/frictions/cf2261g.md`; routed to compiler.
- **Severity:** low

Measurements: native (`cpl build` + g++ -O2) at n = q = 10^6, spec 0.16 s vs AC 0.38 s, outputs equal; emitted C++ 12,263 bytes.
