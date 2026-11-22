// NOIP '20 P3 (Ball Move): sort n stacks of m colored balls (with 1 empty helper stack)
// so each non-empty stack holds one color.
// Divide-and-conquer on color range [ql..=qr]: mark balls > mid as 1, separate every
// column into homogeneous 0/1 halves using the empty stack, then pair same-bottom columns
// to complete pure stacks and recurse on left/right color halves.
// original: corpus/submissions/5445674.cpp

let (n, m): (Int, Int) = read()
domain P = 0..=n
var stks: [P]List<Int> = []
for i in 0..n:
    stks[P(i)] = [read::<Int>() - 1 for _ in 0..m]

var moves: List<(P, P)> = []
var empty = P(n)

fn transfer(u: P, v: P, cnt: Int = 1):
    for _ in 0..cnt:
        moves.push((u, v))
        stks[v].push(stks[u].pop()!)

fn solve_range(cols: List<P>, ql: Int, qr: Int):
    if ql == qr: return
    let mid = (ql + qr) / 2
    fn hi(x: Int) -> Bool = x > mid
    fn cnt_lo(p: P) -> Int = count(not hi(x) for x in stks[p])
    fn pure(p: P) -> Bool:
        let c0 = cnt_lo(p)
        c0 == 0 or c0 == m

    for a in cols:
        let c0 = cnt_lo(a)
        let c1 = m - c0
        let helper = first(p for p in P if p != a and p != empty)!
        transfer(helper, empty, c0)
        for x in stks[a].copy().rev:
            transfer(a, if hi(x): empty else: helper)
        if c0 > c1:
            transfer(helper, a, c0)
            transfer(empty, a, c1)
        else:
            transfer(empty, a, c1)
            transfer(helper, a, c0)
        transfer(empty, helper, c0)

    var cur_cols = cols.copy()
    while true:
        var changed = false
        for i in 0..cur_cols.len:
            for j in 0..cur_cols.len:
                let a = cur_cols[i]
                let b = cur_cols[j]
                if a == b or pure(a) or pure(b) or hi(stks[a][0]) != hi(stks[b][0]): continue
                changed = true
                let bot_hi = hi(stks[a][0])
                let (a0, a1) = if bot_hi: (m - cnt_lo(a), cnt_lo(a)) else: (cnt_lo(a), m - cnt_lo(a))
                let (b0, b1) = if bot_hi: (m - cnt_lo(b), cnt_lo(b)) else: (cnt_lo(b), m - cnt_lo(b))
                transfer(a, empty, a1)
                transfer(b, empty, b1)
                transfer(b, a, m - a0)
                let rem0 = a0 + b0 - m
                let rem1 = a1 + b1
                if rem0 > rem1:
                    transfer(empty, b, rem1)
                else:
                    transfer(b, empty, rem0)
                    swap(mut cur_cols[j], mut empty)
        if not changed: break

    let rest = [a for a in cur_cols if not pure(a)]
    if not rest.is_empty:
        var (a, b) = (rest[0], rest[1])
        if hi(stks[a][0]): swap(mut a, mut b)
        let a1 = m - cnt_lo(a)
        transfer(a, empty, a1)
        transfer(b, a, cnt_lo(b))
        transfer(empty, b, a1)

    let left_cols = [a for a in cur_cols if not hi(stks[a][0])]
    let right_cols = [a for a in cur_cols if hi(stks[a][0])]
    solve_range(left_cols, ql, mid)
    solve_range(right_cols, mid + 1, qr)

solve_range([P(i) for i in 0..n], 0, n - 1)
print(moves.len)
for (u, v) in moves: print(u.to_idx + 1, v.to_idx + 1)
