// Source: Codeforces 2222B (Artistic Balance Tree), https://codeforces.com/contest/2222/problem/B
// limits: 1500 ms, 256 MB
// Problem: an array a of n <= 1e5 values, |a[i]| <= 1e9, and m <= 1e5 operations in order: the
//   i-th first swaps a[u - k] and a[u + k] for k = 1..=y around any center u you choose, then
//   marks the element (not the index) now at index x_i. Print the least possible sum of the
//   unmarked elements. Multitest, sums of n and of m <= 1e5.
// Solution: swaps around a center keep each element on indices of its parity, and within a
//   parity they can reach any order, so an odd x marks an element of the 1-based odd
//   positions and an even x one of the even positions. A class with cnt > 0 marks gets 1 to
//   cnt marked elements of our choice: mark its largest, then each further largest while it
//   is positive and marks remain. Sort each class: O(n log n).
// Bounds: n, m: 1..=200_000; xs: 1..=10 ** 9

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
let xs: [0..m]1..=10 ** 9 = read()

let cnt0 = xs.count(x => x % 2 == 1)
fn best_rem(vals: List<Int>, var cnt: Int) -> Int:
    var s = vals.sorted()
    if cnt > 0:
        cnt -= 1
        s.pop()
        while let x = s.last and cnt > 0 and x > 0:
            cnt -= 1
            s.pop()
    s.sum()

print(best_rem([a[i] for i in 0..n by 2], cnt0) + best_rem([a[i] for i in 1..n by 2], m - cnt0))
