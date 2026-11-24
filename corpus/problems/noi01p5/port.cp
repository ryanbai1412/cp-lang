// Source: NOI 2001 P5 (Equation Solutions), https://dmoj.ca/problem/noi01p5
// Problem: n <= 6 terms (k_i, p_i), M <= 150, p_i >= 1, sum of |k_i M^p_i| < 2^31. Count
//   x in [1..=M]^n with k_1 x_1^p_1 + ... + k_n x_n^p_n = 0.
// Solution: meet in the middle: every value of the first half's sum (at most M^3 = 3.4e6)
//   sorted, then for each sum s of the second half, in order, add the count of -s.
//   O(M^(n/2)) values per half.
// Bounds: n: 1..=6; m: 1..=150; terms.k: -20..=20; terms.p: 1..=3
// original: corpus/submissions/5079272.cpp
let (n, m): (1..=6, 1..=150) = read()
domain T = 0..n
let terms: [T](k: -20..=20, p: 1..=3) = read()

// every value of the sum of k_t x_t^p_t over the terms `ts`
fn sums(ts: List<T>) -> List<Int>:
    var cur = [0]
    for t in ts:
        cur = [s + terms[t].k * x ** terms[t].p for s in cur for x in 1..=m]
    cur

let (left, right) = T.collect().partition(t => Int(t) < n / 2)
let low = sums(left).sorted()
print(sum(low.equal_range(-s).len for s in sums(right).sorted()))
