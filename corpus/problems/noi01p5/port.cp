// Source: NOI 2001 P5 (Equation Solutions), https://dmoj.ca/problem/noi01p5
// Problem: n <= 6 terms (k_i, p_i), M <= 150, p_i >= 1, sum of |k_i M^p_i| < 2^31. Count
//   x in [1..=M]^n with k_1 x_1^p_1 + ... + k_n x_n^p_n = 0.
// Solution: meet in the middle: every value of the first half's sum (at most M^3 = 3.4e6)
//   counted in a hash map, then walk the second half's sums one at a time and look up -s.
//   O(M^(n/2)) expected, storing only the first half.
// Bounds: n: 1..=6; m: 1..=150; terms.k: -2147483647..=2147483647; terms.p: 1..=2147483647
// original: corpus/submissions/5079272.cpp
let (n, m): (1..=6, 1..=150) = read()
domain T = 0..n
type Sum = -2147483647..=2147483647  // every partial sum, as |k_1 M^p_1| + ... < 2^31
let terms: [T](k: -2147483647..=2147483647, p: 1..=2147483647) = read()

// k x^p, with |k| m^p < 2^31: only k = 0 can pair a large p with x > 1
fn term(t: T, x: Int) -> Int = if terms[t].k == 0: 0 else: terms[t].k * x ** terms[t].p
let values = [T] t => [term(t, x) for x in 1..=m]

// every value of the sum of k_t x_t^p_t over the terms `ts`
fn sums(ts: List<T>) -> List<Int>:
    var cur = [0]
    for t in ts:
        cur = [s + v for s in cur for v in values[t]]
    cur

let (left, right) = T.collect().partition(t => Int(t) < n / 2)
let left_sums = sums(left)
var low: Counter<Sum, 0..=3_375_000> = Counter()  // at most M^3 left sums
low.reserve(left_sums.len)
for s in left_sums: low.add(s, 1)

// the right half's sums one at a time, so only the left half is ever stored
fn matches(at: Int, s: Int) -> Int:
    if at == right.len: return low[-s]
    sum(matches(at + 1, s + v) for v in values[right[at]])

print(matches(0, 0))
