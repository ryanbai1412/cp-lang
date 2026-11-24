# Friction Report: ccc05s5

## Counting the keys above a value
- **Where:** `seen.len - seen.rank(s + 1)`.
- **What felt wrong:** "how many earlier scores beat this one" is a suffix count; the port
  spells it as the complement of a rank at `s + 1`, which only works for integer keys.
- **Goal it hurts:** brevity; common mistakes hard to write.
- **Ideal code:** `seen.count_gt(s)` (as `MergeSortTree` has).
- **General fix:** `count_lt/le/gt/ge` on `SortedBag`, matching `MergeSortTree`.
- **Severity:** low

## Dream

```cp
let t: 1..=1e5 = read()
let scores: [t]0..=1e9 = read()
var seen = SortedBag<Int>()
print(fixed(sum(1 + seen.count_gt(s) after seen.add(s) for s in scores), t, 2))
```

## Measurements

- Tokens: port 79, original 413
- Max input (`gen.py --max`): port 0.07 s, 9500 KB, original 0.04 s, 10368 KB
- Emitted C++: 16622 bytes
