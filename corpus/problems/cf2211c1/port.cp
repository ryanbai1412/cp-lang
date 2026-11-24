// Source: Codeforces 2211C1, Equal Multisets (Easy Version),
//   https://codeforces.com/contest/2211/problem/C1
// limits: 2000 ms, 256 MB
// Problem: multitest, sum n <= 2e5; a is a permutation of 1..=n, each b_i is in 1..=n or -1.
//   Can the -1s be filled so every length-k window of b is a rearrangement of the same
//   window of a? Print YES or NO.
// Solution: sliding a window by one forces b_i = a_i outside the middle [n - k, k). Middle
//   positions lie in every window, so the known b there must fit in a's middle multiset:
//   add a_i, subtract b_i, and check no count goes negative. O(n).
// Bounds: n, k: 1..=200_000; a: 1..=n

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=n = read()
let b: [0..n]?Int = read(none: -1)

fn ok() -> Bool:
    for i in 0..n if i < n - k or i >= k:
        if let bi = b[i] and bi != a[i]: return false
    if n - k <= k:
        var cnt: [0..=n]Int = 0
        for i in n - k..k:
            cnt[a[i]] += 1
            if let bi = b[i]: cnt[bi] -= 1
        return cnt.all(v => v >= 0)
    true

print(yes(ok()))
