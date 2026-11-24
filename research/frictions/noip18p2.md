# Friction Report: noip18p2

## 1. `@!multitest` on `fn solve()` checked clean and printed nothing
- **Where:** the v0.2 port.
- **What felt wrong:** the attribute attached to a never-called fn; `cpl check` also missed a type error
  in that body which `cpl build` reports. Repro sent to Core.
- **Goal it hurts:** correct.
- **General fix:** check every fn body; reject `@!multitest` before an item.
- **Severity:** high.

## 2. Unbounded knapsack by hand
- **Where:** `for a in Amount: if let b = Amount.at(a - c) and payable[b]: payable[a] = true`.
- **What felt wrong:** "the amounts payable with these coins" is a std knapsack; the port loops it.
- **Goal it hurts:** express.
- **Ideal code:** `payable.add_coin(c)` on a `Bitset` reachable-sums structure.
- **General fix:** an unbounded-coin reachable-sums helper on `Bitset` (`sums.unbounded(c)`).
- **Severity:** low.

## Measurements
Stress: 30/30 agree with AC 4002899 on gen.py inputs; max (T = 20, n = 100) 0.009 s vs C++ 0.007 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
@!multitest
let coins = read<[read<1..=100>()]1..=25000>()
var payable = Bitset.of(0..=25000, [0])
print(count(for c in coins.sorted(): if not payable[c]: payable.unbounded(c); true))
```
