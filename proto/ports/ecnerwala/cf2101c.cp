// Codeforces 2101 C — 23 Kingdom
//
// Assign each index i a value b[i] in 1..=a[i] to maximize the sum of
// (last(v) - first(v)) over all distinct values v.
//
// By symmetry and Hall's condition, if k values each appear twice, the k
// earliest indices greedily matched to distinct values in 1..=a[i] (using a
// DSU to jump to the largest unused value <= a[i]) give the minimum left
// endpoints `left[0..k]`, and scanning from the right gives the maximum right
// endpoints `right[0..k]`. Pairing `left[t]` with `right[t]` while
// `right[t] > left[t]` maximizes the total span sum(right[t] - left[t]).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()
domain V = 0..=n

fn greedy_picks(xs: [0..n]Int) -> List<Int>:
    var dsu = Dsu<V>()
    var free_val = [V] v => v.ord
    var picks: List<Int> = []
    for i in 0..n:
        let x = dsu.find(V(xs[i]))
        if free_val[x] > 0:
            if let (kept, merged) = dsu.merge(x, V(free_val[x] - 1)):
                free_val[kept] min= free_val[merged]
            picks.push(i)
    picks

let left = greedy_picks([0..n] i => a[i])
let right_rev = greedy_picks([0..n] i => a[n - 1 - i])
let ans = sum(max(0, (n - 1 - r) - l) for (l, r) in zip(left, right_rev))
print(ans)
