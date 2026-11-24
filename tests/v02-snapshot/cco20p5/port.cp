// Source: CCO 2020 P5 (Interval Collection), https://dmoj.ca/problem/cco20p5
// Problem: Q <= 5e5 events add or remove one copy of an interval [l, r] (1 <= l < r <= 1e6). After
//   each, pick a non-empty subset S: prefer one whose intervals share no common interval of
//   positive length, else the smallest such greatest common interval, then the shortest least
//   enclosing interval; print that enclosing length.
// Solution: offline over time: each interval copy lives on a time range, put on O(log Q) nodes of
//   a segment tree over time, walked by DFS with rollback. Along the path keep the shortest
//   interval, the interval with min r (max l on ties) and the one with max l (min r on ties); if
//   min r >= max l, those two give the smallest common part and its enclosing length. Two
//   disjoint intervals (r' <= l) beat that; the best r - l' over such pairs comes from Fenwick
//   prefix-max trees (indexed by r, holding l, and a mirrored one) with undo logs.
//   O(Q log Q log C).
// original: corpus/submissions/4562195.cpp
let q: 1..=500_000 = read()
domain Q = 0..q

var is_add: [Q]Bool = false
var ev_l: [Q]Int = 0
var ev_r: [Q]Int = 0
var max_c = 1

for i in Q:
    let tok: Str = read()
    let l: Int = read()
    let r: Int = read()
    is_add[i] = tok[0] == 'A'
    ev_l[i] = l
    ev_r[i] = r
    max_c max= r + 2

let top = max_c
let width = top + 5
var seg: [0..4 * q + 5]List<Int> = []

fn add_span(o: Int, l: Int, r: Int, ql: Int, qr: Int, v: Int):
    if ql <= l and r <= qr:
        seg[o].push(v)
        return
    let m = (l + r) / 2
    if ql <= m: add_span(o * 2, l, m, ql, qr, v)
    if m < qr: add_span(o * 2 + 1, m + 1, r, ql, qr, v)

var head = HashMap<Int, Int>()
var st_p: List<Int> = []
var st_t: List<Int> = []
var st_v: List<Int> = []
var closed: [Q]Bool = false

for i in Q:
    let ky = ev_l[i] * width + ev_r[i]
    if is_add[i]:
        let nid = st_t.len
        st_p.push(head.get(ky) ?? -1)
        st_t.push(i.ord)
        st_v.push(ky)
        head[ky] = nid
    else:
        let nid = head[ky]
        let s = st_t[nid]
        head[ky] = st_p[nid]
        closed[Q(s)] = true
        if s <= i.ord - 1: add_span(1, 0, q - 1, s, i.ord - 1, ky)

for nid in 0..st_t.len:
    let s = st_t[nid]
    if not closed[Q(s)]: add_span(1, 0, q - 1, s, q - 1, st_v[nid])

let inf = 1_000_000_000
let b = 1_000_005
let w = 2_000_011
let pinf = (2 * b) * w + 2 * b
var bits: [0..2, 0..top + 2]Int = 0
var h_s: List<Int> = []
var h_j: List<Int> = []
var h_v: List<Int> = []

fn ins(s: Int, l: Int, r: Int, cur: Int) -> Int:
    var hi = 0
    var i = l
    while i > 0:
        hi max= bits[s, i]
        i -= i & -i
    var nb = cur
    if hi > 0: nb min= r - hi
    var j = r
    while j <= top:
        if l > bits[s, j]:
            h_s.push(s)
            h_j.push(j)
            h_v.push(bits[s, j])
            bits[s, j] = l
        j += j & -j
    nb

var ans: [Q]Int = inf

fn dfs(o: Int, l: Int, r: Int, dj0: Int, dj1: Int, p0: Int, p1: Int, ml: Int):
    let mark = h_s.len
    var d0 = dj0
    var d1 = dj1
    var b0 = p0
    var b1 = p1
    var b2 = ml
    for k in 0..seg[o].len:
        let v = seg[o][k]
        let cl = v / width
        let cr = v % width
        d0 = ins(0, cl, cr, d0)
        d1 = ins(1, top - cr, top - cl, d1)
        b0 min= (cr + b) * w + (-cl + b)
        b1 min= (-cl + b) * w + (cr + b)
        b2 min= cr - cl
    if l == r:
        var bi = b2
        var bu = b2
        let r1 = b0 / w - b
        let l1 = -(b0 % w - b)
        let l2 = -(b1 / w - b)
        let r2 = b1 % w - b
        if r1 >= l2:
            let ci = r1 - l2
            let cu = r2 - l1
            if ci < bi or (ci == bi and cu < bu):
                bi = ci
                bu = cu
        let d = min(d0, d1)
        if d < inf and (0 < bi or (0 == bi and d < bu)):
            bu = d
        ans[Q(l)] = bu
    else:
        let m = (l + r) / 2
        dfs(o * 2, l, m, d0, d1, b0, b1, b2)
        dfs(o * 2 + 1, m + 1, r, d0, d1, b0, b1, b2)
    while h_s.len > mark:
        let idx = h_s.len - 1
        bits[h_s[idx], h_j[idx]] = h_v[idx]
        h_s.pop()
        h_j.pop()
        h_v.pop()

dfs(1, 0, q - 1, inf, inf, pinf, pinf, inf)
for i in Q: print(ans[i])
