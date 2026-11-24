# abc480f friction

There is no `dream.cp`; the ideal is below.

## Dream
```cp
let (n, m): (1..=20, 1..=100) = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let t = a.sum() / m  // each person's share
let rem: [Subset<I>]Int = [Subset<I>] s => sum(a[i] for i in s) % t
let saved: [Subset<I>]Int = dp(s => if s.is_empty: 0 else: max(saved[s.without(i)] for i in s)! + Int(rem[s] == 0))
print(m - saved[Subset<I>.full])
```

## A subset sum built by a hand recurrence
- **Where:** `port.cp`, `rem`: `if let i = s.first(): rem[s] = (rem[s.without(i)] + a[i]) % t`.
- **What felt wrong:** it is the subset sum mod `t`, but spelled as a recurrence on the lowest
  member so that it runs in O(2^N) instead of O(2^N N).
- **Goal it hurts:** express.
- **Ideal code:** `[Subset<I>] s => sum(a[i] for i in s) % t`, at the recurrence's cost.
- **General fix:** `std/subset.cp` has membership, `with`/`without`, `first`, `items`,
  `subsets` and `supersets`, but no sum over members. Add a std subset-sum table (`Subset<I>.sums(a)`, built in O(2^N)), which every
  bitmask DP over sums would use.
- **Severity:** low.
- **Owner:** Content (stdlib `Subset`).

## Measurements
- Tokens: port 181, original 236.
- `--max` (best of 10): port 0.020 s / 19 MB, `orig.cpp` (not judge-accepted, stress-checked)
  0.059 s / 14 MB. Same output.
- Emitted C++: 17,938 bytes.
