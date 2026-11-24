# btoi19p4 frictions

## Dream

```cp
let variety = knapsack_max(hours, weight: b => b, value: b => min(b, n), upto: Paid)
print(Paid.find(p => p >= work and variety[p] >= Some(n * k)).map(p => p - work) ?? "Impossible")
```

## Optional order

`?Int` values have no order, so the reachable-set test reads `(variety[p] ?? 0) >= n * k`
(a reachable set's variety is at least 0 and n * k is at least 1). std has no 0/1 knapsack
that keeps the best value per exact weight.
