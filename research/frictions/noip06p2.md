# Friction Report: noip06p2

## 1. The input's 0 means "no owner"
- **Where:** `main: 0..=n` read, then `Item.at(items[i].main)`.
- **What felt wrong:** the statement's sentinel 0 has to be read as an `Int` and converted; the read cannot
  say "an `?Item`, 0 for None".
- **Goal it hurts:** types carry the proofs.
- **Ideal code:** `let items: [Item](price: 0..=10000, weight: 1..=5, main: ?Item @zero_none) = read()`.
- **General fix:** a read form for `?D` where a stated value (0, -1) is None.
- **Severity:** low.

## Repro: the 0-for-None read

On a fresh `cpl run`, `main: ?Item @zero_none` is `error[E0103]: expected `)`, found `@``, and a
plain `main: ?Item` rejects the input's 0: `error[R0102]: 0 is not in `Item`, which is 1..=3`.

## 2. Accessory subsets and the grouped knapsack by hand

- **Where:** `let plain = bundles.len` / `for t in 0..plain: bundles.push(...)`, then
  `for j in Money.rev: for b in bundles: if let rest = Money.at(j - b.cost): best[j] max= ...`.
- **What felt wrong:** "every subset of the accessories" and "take at most one option per group"
  are both named ideas; the port writes the C++ doubling and the triple loop.
- **Repro:** on a fresh `cpl run`, `[1, 2].subsets()` is `error[E0203]: a `List<Int>` has no
  method `subsets`` (std has `Subset.subsets` only), and `best.knapsack_group(bundles)` on a
  `[M]Int` is `error[E0203]: a `[M]Int` has no method `knapsack_group``.
- **Ideal code:** the `## Dream` lines `subsets(extras[i]).map(...)` and `best.knapsack_group(bundles)`.
- **General fix:** `List.subsets` (small lists) and a grouped 0/1 knapsack step in `std.dp`.
- **Owner:** stdlib. **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5165705; max (n = 32000, m = 60) 0.006 s vs C++ 0.019 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let items = read<[Item](price: 0..=10000, weight: 1..=5, main: ?Item @zero_none)>()
let extras = Item.group(i => items[i].main)
var best: [Money]Int = 0
for i in Item if items[i].main == None:
    let bundles = subsets(extras[i]).map(es => (cost: sum(items[e].price for e in [i] + es), worth: sum(worth(e) for e in [i] + es)))
    best.knapsack_group(bundles)
print(best[Money.last!])
```
