# Friction: `sac22cc2p5`

## A `[Bool]` comprehension binds its index as an `Int`

- **Where:** `spec.cp:7-8`, `fn half_of(p: Bool) -> Fenwick<I, Int> = ...` and `var half: [Bool]Fenwick<I, Int> = [half_of(false), half_of(true)]`.
- **What felt wrong:** the table is keyed by `Bool` and indexed `half[odd(i)]`, but `[Bool] p => ... odd(i) == p ...` is E0302 "can't compare a `Bool` with a `Int`", so the two entries are built by a named fn called twice, in the literal's `false, true` order.
- **Goal it hurts:** coherence, express
- **Ideal code:** `var half = [Bool] p => Fenwick([I] i => if odd(i) == p: a[i] else: 0)`
- **General fix:** a comprehension over a key type binds its parameter at that key type (`[Bool] p` gives `p: Bool`, as `[Dir] d` should give `d: Dir`). Repro: `let x = [Bool] p => if p: 1 else: 0` is E0302 "expected `Bool`, found `Int`". Same gap as `research/frictions/cf2261g.md`; routed to compiler.
- **Severity:** low

Measurements: native (`cpl build` + g++ -O2) at n = q = 10^6, spec 0.16 s vs AC 0.38 s, outputs equal; emitted C++ 12,263 bytes.

## Two Fenwick trees picked by parity instead of a strided view
- **Where:** `port.cp`, `domain Parity`, `fn side`, and each tree built with zeros at the other parity.
- **What felt wrong:** "every second element of [L, R]" is a strided range; the port spends half
  of each tree on zeros and names the parity by hand.
- **General fix:** a step on `Range<D>` (`(l..=r).step(2)`) that a `Fenwick` or `SegTree` fold accepts, backed by per-residue trees.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

```cp
let n = read<1..=1e6>()
let q = read<1..=1e6>()
domain Pos = 1..=n
var a = Fenwick(read<[Pos]-1e9..=1e9>(), stride: 2)
for _ in 0..q:
    match read<1..=2>():
        1 => a.set(read<Pos>(), read<-1e9..=1e9>())
        2 =>
            let (l, r) = read<(Pos, Pos)>()
            print(a.fold((l..=r).step(2)))
```
