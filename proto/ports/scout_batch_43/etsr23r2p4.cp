// ETSR '23 Round 2 P4 — Sliding-Window Bitmask DP (CP Lang v2 spec)
// original: corpus/submissions/5494510.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Order items 0..n minimizing adjacent XOR cost subject to tolerance limits
// (each item i allows at most tol[i] later items to precede it).

@multitest
let n: 1..=1000 = read()
domain Item = 0..n
domain Win = 0..15
let items: [Item](val: Int, tol: 0..=7) = read()

// FRICTION ET3: `@memo(hash)` sits inside `@multitest` scope and depends on
// per-case `n` and `items`. The v2 spec should state explicitly that `@memo`
// caches inside `@multitest` clear automatically between test cases.
@memo(hash)
fn dp(ind: Int, prv: ?Item, used: Subset<Win>) -> Int:
    if ind >= n: return 0
    if Win.first in used:
        return dp(ind + 1, prv, used.shifted(-1))
    let limit = min(
        ind + w.to_idx + items[Item(ind + w.to_idx)].tol
        for w in Win
        if ind + w.to_idx < n and w not in used
    ) ?? (n - 1)
    min(
        (if let p = prv: items[p].val ^ items[Item(i)].val else: 0)
            + dp(ind, Item(i), used.with(Win(i - ind)))
        for i in ind..=min(limit, n - 1)
        if Win(i - ind) not in used
    )!

print(dp(0, none, Subset<Win>.first))
