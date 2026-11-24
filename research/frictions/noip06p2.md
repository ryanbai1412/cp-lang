# Friction Report: noip06p2

## 1. The input's 0 means "no owner"
- **Where:** `main: 0..=n` read, then `Item.at(items[i].main)`.
- **What felt wrong:** the statement's sentinel 0 has to be read as an `Int` and converted; the read cannot
  say "an `?Item`, 0 for None".
- **Goal it hurts:** types carry the proofs.
- **Ideal code:** `let items: [Item](price: 0..=10000, weight: 1..=5, main: ?Item @zero_none) = read()`.
- **General fix:** a read form for `?D` where a stated value (0, -1) is None.
- **Severity:** low.

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
