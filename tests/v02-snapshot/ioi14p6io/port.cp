// Source: IOI 2014 P6 (Holiday, standard I/O), https://dmoj.ca/problem/ioi14p6io
// Problem: n <= 1e5 cities on a line with attraction counts <= 1e9, a start city and
//   d <= 2n + n/2 days; each day moves to a neighbour or visits the current city (each city
//   counts once). Print the most attractions visitable.
// Solution: an optimal plan goes out one side, back, then out the other. Per side,
//   f(t) = best with t days there: pick the far end i, then the top t - c(i + 1) cities among
//   the first i + 1 (c = 2 for the side done first, 1 for the other). The best i is monotone
//   in t, so divide and conquer on t; top-k of a prefix comes from blocks of 300 with sorted
//   prefixes and prefix sums. Combine the sides over the day split, with or without the start
//   city, both orientations: O((n + d) log d) top-k queries of O(300) each.
// Stdin: n, start, d, then n attraction values. Prints maximum sum.
// Uses top-level recursive fn (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/4691224.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

var g_sorted_pref: List<List<Int>> = []
var g_psum: List<List<Int>> = []
var g_gaps: List<List<(Int, Int)>> = []
var g_ans: List<Int> = []
var g_mul = 1

fn topk_query(i: Int, k: Int) -> Int:
    if k <= 0: return 0
    var rem = min(k, i + 1)
    let b = i / 300
    var tot = 0
    let sp = g_sorted_pref[b]
    for pair in g_gaps[b]:
        if rem == 0: break
        let x = -pair.0
        let j = pair.1
        if j <= i:
            if rem - 1 >= sp.len or x > sp[rem - 1]:
                rem -= 1
                tot += x
            else:
                break
    return tot + g_psum[b][rem]

fn dnc_rec(l: Int, r: Int, ql: Int, qr: Int):
    if l > r: return
    let mid = (l + r) / 2
    var best = -1
    var opt = ql
    for i in ql..=qr:
        let v = topk_query(i, mid - g_mul * (i + 1))
        if v > best:
            best = v
            opt = i
    g_ans[mid] = max(0, best)
    dnc_rec(l, mid - 1, ql, opt)
    dnc_rec(mid + 1, r, opt, qr)

fn solve_side(mul: Int, d: Int, a: List<Int>) -> List<Int>:
    let n = a.len
    g_ans = [0..d + 1] _ => 0
    if n == 0: return g_ans
    g_mul = mul
    g_sorted_pref = []
    g_psum = []
    g_gaps = []
    var ind = 0
    while ind <= n:
        var s: List<Int> = [-a[i] for i in 0..ind]
        s.sort()
        var sp: List<Int> = []
        var ps: List<Int> = [0]
        for x in s:
            let v = -x
            sp.push(v)
            ps.push(ps.last! + v)
        g_sorted_pref.push(sp)
        g_psum.push(ps)
        let hi = min(n, ind + 300)
        var g: List<(Int, Int)> = [(-a[j], j) for j in ind..hi]
        g.sort()
        g_gaps.push(g)
        ind += 300
    dnc_rec(0, d, 0, n - 1)
    return g_ans

fn one_way(n: Int, start: Int, d: Int, a: List<Int>) -> Int:
    var left_a: List<Int> = []
    var i = start - 1
    while i >= 0:
        left_a.push(a[i])
        i -= 1
    var right_a: List<Int> = [a[j] for j in start + 1..n]
    let ldp = solve_side(2, d, left_a)
    let rdp = solve_side(1, d, right_a)
    var best = 0
    for t in 0..=d:
        best max= ldp[t] + rdp[d - t]
        if t < d: best max= a[start] + ldp[t] + rdp[d - 1 - t]
    return best

let n: Int = read()
let start: Int = read()
let d: Int = read()
var a: List<Int> = [ri() for _ in 0..n]
var rev_a: List<Int> = []
var ii = n - 1
while ii >= 0:
    rev_a.push(a[ii])
    ii -= 1
print(max(one_way(n, start, d, a), one_way(n, n - 1 - start, d, rev_a)))
