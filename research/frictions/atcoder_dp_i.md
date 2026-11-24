# atcoder_dp_i — Coins

chance[h] is the probability of h heads so far; each coin shifts mass p up one step and keeps
1 - p, updated in place going down over h. Sum over the majority counts.

## Dream

```cp
// Source: AtCoder Educational DP Contest I (Coins), https://atcoder.jp/contests/dp/tasks/dp_i
// Problem: N coins, N odd and <= 2999; coin i shows heads with probability p_i, 0 < p_i < 1.
//   Print the probability of more heads than tails, to absolute error 1e-9.
// Solution: chance[h] is the probability of h heads among the coins tossed so far. Each coin
//   moves chance[h] * p up to h + 1 and keeps chance[h] * (1 - p); going down over h updates
//   in place. Sum chance[h] over 2h > N. O(N^2).
let n: Odd<1..=2999> = read()
let heads: [n]0.0<..<1.0 = read()

domain Heads = 0..=n
var chance: [Heads]F64 = 0.0
chance[Heads.first!] = 1.0
for p in heads:
    for h in Heads.rev:
        if let up = h.next: chance[up] += chance[h] * p
        chance[h] *= 1.0 - p
print(sum(chance[h] for h in Heads if 2 * h.label > n), precision: 10)
```

## Probabilities have no bounded real type
- **Where:** `port.cp`, line 9: `let heads: [n]F64 = read()`.
- **What felt wrong:** the statement says 0 < p_i < 1, but a real range does not parse as a
  type, so the bound is lost and `1.0 - p` is not known to be a probability.
  Repro: `let p: 0.0<..<1.0 = read()` gives `error[E0105]: expected a type`.
- **Goal it hurts:** types carry the proofs (statement bounds as types).
- **Ideal code:** `let heads: [n]0.0<..<1.0 = read()`.
- **General fix (types):** range types over F64 (inclusive and exclusive ends), checked at read.
- **Severity:** low.

## Parity of n is not a type
- **Where:** `port.cp`, line 8: `let n: 1..=2999 = read()`.
- **What felt wrong:** "N is odd" is what makes "more heads than tails" never a tie; it can
  only live in the header. Repro: `let n: Odd<1..=2999> = read()` gives
  `error[E0204]: no type or domain named \`Odd\`, and not in the stdlib (yet)`.
- **Goal it hurts:** types carry the proofs.
- **Ideal code:** `let n: Odd<1..=2999> = read()`, or a `where n % 2 == 1` refinement checked at read.
- **General fix (types):** read-time refinements on integer bounds beyond intervals.
- **Severity:** low.
