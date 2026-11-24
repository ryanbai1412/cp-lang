// Source: Codeforces 2124H (Longest Good Subsequence),
//   https://codeforces.com/contest/2124/problem/H
// limits: 7000 ms, 1024 MB
// Problem: b of length m is good if 1 <= b[i] <= i and some permutation p has, for every i, b[i]
//   the smallest index with min(p[b[i]..=i]) = p[i]. Given a (n <= 15000, 1 <= a[i] <= n),
//   print the length of its longest good subsequence. Multitest, sum of n^2 <= 15000^2.
// Solution: a good sequence is a block: after its start value v, each element is v again (one
//   longer) or a w with v < w <= length + 1 that opens a nested block ending at w again. With
//   a sentinel 0 in front, max_len[i, j] (a[j] = a[i]) is the longest such block from i to j.
//   Fill i right to left, scanning j with the running length cur_best: a[j] = a[i] adds one,
//   a[i] < a[j] <= cur_best + 1 jumps to max_len[first such a[j] after i, j]. The answer is
//   cur_best at the sentinel. O(n^2) time, a triangle of 16-bit entries.
// Bounds: n0: 1..=15_000; raw: 1..=n0

fn tidx(i: Int, j: Int) -> Int = j * (j + 1) / 2 + i

@!multitest
let n0: 1..=15_000 = read()
let raw: [0..n0]1..=n0 = read()
let n = n0 + 1
let a = [0..n] i => if i == 0: 0 else: raw[i - 1]
var max_len: [0..n * (n + 1) / 2]0..=2 * n = 0

for i in (0..n).rev:
    let ai = a[i]
    var first_vis: [0..=n]?Int = None
    var cur_best = ai - 1
    for j in i..n:
        let aj = a[j]
        if aj < ai: continue
        if aj == ai:
            cur_best += 1
            max_len[tidx(i, j)] = cur_best
        elif aj <= cur_best + 1:
            first_vis[aj] ??= j
            if let fv = first_vis[aj]:
                cur_best max= max_len[tidx(fv, j)]
    if i == 0: print(cur_best)
