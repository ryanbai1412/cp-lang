// Source: Codeforces 2101C, 23 Kingdom, https://codeforces.com/contest/2101/problem/C
// Problem: multitest, sum n <= 2e5, a_i in 1..=n. Choose b_i in 1..=a_i to maximize the
//   beauty: the sum over distinct values v in b of (last index of v - first index of v).
// Solution: give the earliest indices the largest still-free value <= a[i] (a SortedSet of
//   free values); the t-th index that gets one is the best t-th left end. The same scan from
//   the right gives the right ends. Pair the t-th left with the t-th right and add the
//   spans while positive. O(n log n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()
domain V = 1..=n

fn reflect(i: Int) -> Int = n - 1 - i

fn greedy_picks(xs: [0..n]1..=n) -> List<Int>:
    var free = SortedSet<V>.of(V.collect())
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
