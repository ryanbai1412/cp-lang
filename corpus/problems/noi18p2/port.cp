// Source: NOI 2018 Day 1 P2 (Inverse), https://dmoj.ca/problem/noi18p2
// Problem: p is good if bubble sort makes exactly 1/2 sum |i - p_i| swaps. Given a permutation q
//   of 1..n, count the good permutations of length n lexicographically greater than q, mod
//   998244353. T = 5 tests, n <= 6e5, sum n <= 2e6.
// Solution: p is good iff it has no decreasing subsequence of length 3: each element is a new
//   prefix maximum or the smallest unused value. Such suffixes are ballot paths, counted after
//   i placed values with maximum j by C(2n-i-j, n-i) - C(2n-i-j, n-j-1). Walk q: where p first
//   exceeds q at i, p_i is a new maximum >= max(mx, q_i) + 1, and those completions sum to one
//   ballot term; stop once q's own prefix is not good. O(n) per test after factorials.
// Runs on proto/cp.py lib-run today.

type Z = Mod<998244353>

let t: Int = read()
var cases: List<List<Int>> = []
var max_n = 1
for _ in 0..t:
    let n: Int = read()
    if n > max_n: max_n = n
    var q: List<Int> = []
    for _ in 0..n:
        let v: Int = read()
        q.push(v)
    cases.push(q)

let comb = Comb<Z>(2 * max_n + 5)

fn safe_choose(n: Int, k: Int) -> Z:
    if n < 0 or k < 0 or k > n: return 0
    comb.choose(n, k)

fn ballot(n: Int, i: Int, low: Int) -> Z:
    if low > n: return 0
    let steps = 2 * n - i - low
    safe_choose(steps, n - i) - safe_choose(steps, n - low - 2)

for c in 0..t:
    let q = cases[c]
    let n = q.len
    var seen = List.filled(n + 2, false)
    var min_unused = 1
    var mx = 0
    var ans: Z = 0

    for i in 1..=n:
        let v = q[i - 1]
        let low = max(mx, v) + 1
        ans += ballot(n, i, low)

        if v > mx:
            mx = v
        elif v != min_unused:
            break

        seen[v] = true
        while min_unused <= n and seen[min_unused]:
            min_unused += 1

    print(ans)
