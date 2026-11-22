// Codeforces 2101 C — 23 Kingdom
//
// Assign each index `i` a value `b[i] in 1..=a[i]` to maximize the sum of
// `(last(v) - first(v))` over all distinct values `v`.
//
// By symmetry and Hall's condition, if `k` values each appear twice, the `k`
// earliest indices greedily matched to the largest still-free value in `1..=a[i]`
// give the minimum left endpoints `left[0..k]`, and scanning the reflected
// array gives the maximum right endpoints `reflect(right_rev[0..k])`. Pairing
// `left[t]` with `reflect(right_rev[t])` while the span is positive maximizes
// `sum(reflect(r) - l)`.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()
domain V = 1..=n

fn reflect(i: Int) -> Int = n - 1 - i

fn greedy_picks(xs: [0..n]1..=n) -> List<Int>:
    var free = SortedSet<V>.of([v for v in V])
    var picks: List<Int> = []
    for (i, x) in xs.items:
        if let v = free.range(..=V(x)).last:
            free.remove(v)
            picks.push(i)
    picks

let left = greedy_picks(a)
let right_rev = greedy_picks([0..n] i => a[reflect(i)])
let ans = sum(max(0, reflect(r) - l) for (l, r) in zip(left, right_rev))
print(ans)
