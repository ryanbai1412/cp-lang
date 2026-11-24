// Source: Mock CCC '23 Contest 1 J3 (Pairing Gifts), https://dmoj.ca/problem/mccc5j3
// Problem: n <= 2e5 distinct prices a_i, n distinct prices b_j, all in 1..=M (M <= 1e18), and
//   0 <= K < n. Print the most disjoint pairs (i, j) with a_i + b_j = M and |i - j| >= K.
// Solution: the a_i are distinct, so b_j has at most one partner a_i = M - b_j, and the b_j are
//   distinct, so the good pairs are already disjoint: count them. A Map takes each value of a
//   to its positions; for each j look up M - b_j and count the positions <= j - K or >= j + K
//   by binary search (all of them when K = 0). O(n log n).
// original: corpus/submissions/5333116.cpp

let (n, k, m): (Int, Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()
let b: [I]Int = read()

var pos = Map<Int, List<Int>>()
for i in I:
    let x = a[i]
    if x not in pos: pos[x] = []
    pos[x].push(i.ord)

fn first_ge(xs: List<Int>, target: Int) -> Int:
    let lo = xs.first_ge(target) ?? xs.len
    lo

var ans = 0
for i in I:
    let need = m - b[i]
    if let xs = pos.get(need):
        if k == 0:
            ans += xs.len
        else:
            let idx = i.ord
            ans += first_ge(xs, idx - k + 1)
            ans += xs.len - first_ge(xs, idx + k)

print(ans)
