// SAC '22 CC4 P4 — original: corpus/submissions/4411200.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)
// Rethought from O(n log n) custom LazySegTree (280 LOC) to O(n) MonoDeque two-pointers.

let (n, ql, qr): (1..=200_000, 0..=1_000_000_000, 0..=1_000_000_000) = read()
let a: [0..n]Int = read()

fn count_at_most(limit: Int) -> Int:
    if limit < 0:
        return 0
    var lo = MonoDeque<Int>(min)
    var hi = MonoDeque<Int>(max)
    var l = 0
    var ans = 0
    for r in 0..n:
        lo.push(at: r, val: a[r])
        hi.push(at: r, val: a[r])
        // FRICTION 39.7: hi.best! - lo.best! needs double `!` despite l <= r invariant
        while hi.best! - lo.best! > limit:
            l += 1
            lo.drop_before(l)
            hi.drop_before(l)
        ans += r - l + 1
    ans

print(count_at_most(qr) - count_at_most(ql - 1))
