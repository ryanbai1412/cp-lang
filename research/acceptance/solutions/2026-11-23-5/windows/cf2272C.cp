// C. Yet Another Black Cell Problem
// https://codeforces.com/contest/2272/problem/C
// for a given k, a cell with no given neighbour within k needs an extra cell beside it; two such
// consecutive cells at most 2k apart can share one. Binary search k.
@!multitest
let (n, x): (2..=2e5, 0..=n) = read()
let a: [0..n]0..=1e9 = read()

fn isolated(i: Int, k: Int) -> Bool = (i == 0 or a[i] - a[i - 1] > k) and (i == n - 1 or a[i + 1] - a[i] > k)

fn need(k: Int) -> Int:
    var cnt = 0
    var i = 0
    while i < n:
        if isolated(i, k):
            cnt += 1
            if i + 1 < n and isolated(i + 1, k) and a[i + 1] - a[i] <= 2 * k:
                i += 1
        i += 1
    cnt

print((1..=1e9).first_true(k => need(k) <= x)!)
