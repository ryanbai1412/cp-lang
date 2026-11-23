// Codeforces 2101 C — 23 Kingdom, tuned: port.cp's greedy with the free
// values kept as path-halving links (orig.cpp's union-find) instead of a
// SortedSet. up[v] leads to the largest free value at most v; 0 is none.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()

fn reflect(i: Int) -> Int = n - 1 - i

fn greedy_picks(xs: [0..n]1..=n) -> List<Int>:
    var up: [0..=n]0..=n = [0..=n] v => v
    var picks: List<Int> = []
    for (i, x) in xs.items:
        var v: 0..=n = x
        while up[v] != v:
            up[v] = up[up[v]]
            v = up[v]
        if v > 0:
            up[v] = v - 1
            picks.push(i)
    picks

let left = greedy_picks(a)
let right_rev = greedy_picks([0..n] i => a[reflect(i)])
let ans = sum(max(0, reflect(r) - l) for (l, r) in zip(left, right_rev))
print(ans)
