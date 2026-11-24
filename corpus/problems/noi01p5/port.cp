// Source: NOI 2001 P5 (Equation Solutions), https://dmoj.ca/problem/noi01p5
// Problem: n <= 6 terms (k_i, p_i), M <= 150, p_i >= 1, sum of |k_i M^p_i| < 2^31. Count
//   x in [1..=M]^n with k_1 x_1^p_1 + ... + k_n x_n^p_n = 0.
// Solution: meet in the middle: every value of the first half's sum (at most M^3 = 3.4e6)
//   into a counts map, then for each sum s of the second half add the count of -s.
//   O(M^(n/2)) values per half.
// Bounds: n: 1..=6; m: 1..=150; terms.k: -20..=20; terms.p: 1..=3
let n: 1..=6 = read()
let m: 1..=150 = read()
domain T = 0..n
let terms: [T](k: -20..=20, p: 1..=3) = read()

fn pow(x: 1..=150, p: 1..=3) -> 1..=3375000:
    x ** p

// Every value of sum k_t * x_t^p_t over the given terms.
fn sums(ts: List<T>) -> List<Int>:
    var cur: List<Int> = [0]
    for t in ts:
        cur = [s + terms[t].k * x ** terms[t].p for s in cur for x in 1..=m]
    cur

let left = [t for t in T if t.ord < n / 2]
let right = [t for t in T if t.ord >= n / 2]
let seen = counts(sums(left))
print(sum(seen[-s] for s in sums(right)))
