// Source: CCC 2023 Senior S2 (Symmetric Mountains), https://dmoj.ca/problem/ccc23s2
// Problem: n <= 5000 heights 0 <= h_i <= 1e5. A crop h[l..=r] has asymmetric value the sum of
//   |h[l + i] - h[r - i]| over the pairs from the outside in. For every length 1..=n print the
//   least asymmetric value of a crop of that length.
// Solution: grow every crop from its centre: odd (i) and even (i, i + 1) centres extend one pair
//   at a time, adding |h[i - j] - h[i + j + off]|, and the length 2j + 1 + off keeps the minimum.
//   O(n^2).
// original: corpus/submissions/5350805.cpp

fn main_solve():
    let n: Int = read()
    domain I = 0..n
    let h: [I]Int = read()
    domain L = 0..n
    var ans: [L]?Int = None
    for i in 0..n:
        for off in 0..=1:
            var cur = 0
            var j = 0
            while i - j >= 0 and i + j + off < n:
                cur += abs(h[I(i - j)] - h[I(i + j + off)])
                ans[L(2 * j + off)] min= cur
                j += 1
    var out: List<Int> = [ans[k]! for k in L]
    print(out)

main_solve()
