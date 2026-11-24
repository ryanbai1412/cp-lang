// Source: NOIP 2020 P3 (Ball Move), https://dmoj.ca/problem/noip20p3
// limits: 1000 ms, 512 MB
// Problem: n + 1 pillars of capacity m (2 <= n <= 50, 2 <= m <= 400); pillars 1..n each hold
//   m balls and pillar n + 1 is empty; each of the n colours has m balls. A move takes the top
//   ball of one pillar to another that is not full. Print any plan of at most 820000 moves that
//   gathers every colour on one pillar: the count, then each move as (from, to).
// Solution: divide and conquer on colours: solve(pillars, lo..hi) splits at mid into small and
//   big colours. First each pillar is sorted into a small block and a big block using a helper
//   pillar and the empty one; then two mixed pillars with the same bottom kind are merged so one
//   becomes pure (the empty pillar may change), until at most two mixed ones are left and one
//   swap fixes them. Recurse on the pillars by bottom kind. O(n m log n) moves.
// Uses top-level recursive fn (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/5445674.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

var g_n = 0
var g_m = 0
var g_stks: List<List<Int>> = []
var g_mu: List<Int> = []
var g_mv: List<Int> = []
var g_empty = 0

fn transfer(u: Int, v: Int, cnt: Int):
    for _ in 0..cnt:
        g_mu.push(u)
        g_mv.push(v)
        let x = g_stks[u].pop()!
        g_stks[v].push(x)

fn cnt_lo(p: Int, mid: Int) -> Int:
    var c = 0
    for x in g_stks[p]:
        if x <= mid: c += 1
    return c

fn is_pure(p: Int, mid: Int) -> Bool:
    let c0 = cnt_lo(p, mid)
    return c0 == 0 or c0 == g_m

fn solve_range(cols: List<Int>, ql: Int, qr: Int):
    if ql == qr: return
    let mid = (ql + qr) / 2
    for a in cols:
        let c0 = cnt_lo(a, mid)
        let c1 = g_m - c0
        var helper = 0
        for p in 0..=g_n:
            if p != a and p != g_empty:
                helper = p
                break
        transfer(helper, g_empty, c0)
        var idx = g_m - 1
        while idx >= 0:
            if g_stks[a][idx] > mid: transfer(a, g_empty, 1)
            else: transfer(a, helper, 1)
            idx -= 1
        if c0 > c1:
            transfer(helper, a, c0)
            transfer(g_empty, a, c1)
        else:
            transfer(g_empty, a, c1)
            transfer(helper, a, c0)
        transfer(g_empty, helper, c0)

    var cur_cols: List<Int> = cols.collect()

    while true:
        var changed = false
        for i in 0..cur_cols.len:
            for j in 0..cur_cols.len:
                let a = cur_cols[i]
                let b = cur_cols[j]
                if a == b or is_pure(a, mid) or is_pure(b, mid): continue
                let bot_a = g_stks[a][0] > mid
                let bot_b = g_stks[b][0] > mid
                if bot_a != bot_b: continue
                changed = true
                let a0 = if bot_a: g_m - cnt_lo(a, mid) else: cnt_lo(a, mid)
                let a1 = g_m - a0
                let b0 = if bot_a: g_m - cnt_lo(b, mid) else: cnt_lo(b, mid)
                let b1 = g_m - b0
                transfer(a, g_empty, a1)
                transfer(b, g_empty, b1)
                transfer(b, a, g_m - a0)
                let rem0 = a0 + b0 - g_m
                let rem1 = a1 + b1
                if rem0 > rem1:
                    transfer(g_empty, b, rem1)
                else:
                    transfer(b, g_empty, rem0)
                    swap(mut cur_cols[j], mut g_empty)
        if not changed: break

    var rest: List<Int> = []
    for a in cur_cols:
        if not is_pure(a, mid): rest.push(a)
    if rest.len > 0:
        var a = rest[0]
        var b = rest[1]
        if g_stks[a][0] > mid:
            swap(mut a, mut b)
        let a1 = g_m - cnt_lo(a, mid)
        transfer(a, g_empty, a1)
        transfer(b, a, cnt_lo(b, mid))
        transfer(g_empty, b, a1)

    var left_cols: List<Int> = []
    var right_cols: List<Int> = []
    for a in cur_cols:
        if g_stks[a][0] <= mid: left_cols.push(a)
        else: right_cols.push(a)
    solve_range(left_cols, ql, mid)
    solve_range(right_cols, mid + 1, qr)

g_n = ri()
g_m = ri()
for _ in 0..g_n + 1: g_stks.push([])
for i in 0..g_n:
    for _ in 0..g_m: g_stks[i].push(ri() - 1)
g_empty = g_n
var init_cols: List<Int> = [i for i in 0..g_n]
solve_range(init_cols, 0, g_n - 1)
print(g_mu.len)
for i in 0..g_mu.len: print(g_mu[i] + 1, g_mv[i] + 1)
