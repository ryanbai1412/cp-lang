// Source: CCO 2005 P4 (Primed Sequences), https://dmoj.ca/problem/cco05p4
// Problem: t <= 20 tests, each a sequence of n <= 1e4 positive integers <= 1e4. Print the
//   shortest run of consecutive elements, length >= 2, whose sum is prime (leftmost on ties),
//   as "Shortest primed subsequence is length x: ..." followed by its elements, or print
//   "This sequence is anti-primed." if there is none. Multitest.
// Solution: brute force over prefix sums: try lengths 2, 3, ... and for each every left end
//   in order, testing the window sum with is_prime; the first hit is the answer. O(n^2)
//   windows in the worst case, each with a primality test of a sum <= 1e8.
// original: corpus/submissions/4665767.cpp
use std.math.{is_prime}

@multitest
let n: Int = read()
let a: [0..n]Int = read()
let pre = prefix([a[i] for i in 0..n])

for len in 2..=n:
    for l in 0..=n - len:
        if is_prime(pre.cut(l + len) - pre.cut(l)):
            print("Shortest primed subsequence is length {len}:", [a[i] for i in l..l + len])
            return
print("This sequence is anti-primed.")
