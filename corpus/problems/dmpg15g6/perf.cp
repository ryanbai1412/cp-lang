// Tuned: the Misra-Gries summaries are three (flavour, count) slots per node in two flat
// lists of a bottom-up segment tree, joined in place through a six-slot scratch, as the
// accepted C++'s fixed arrays are, instead of a List per node and per join.
//
// Source: DMPG '15 G6 (Yōkan), https://dmoj.ca/problem/dmpg15g6
// Problem: n <= 2e5 pieces with flavours in 1..=m, q <= 2e5 queries l r: can two people each
//   get >= L/3 pieces (L = r - l + 1) of one flavour, the same or two different? Print YES or
//   NO per query.
// Solution: YES iff the top count c1 >= 2L/3, or the top two counts are both >= L/3. A
//   flavour with >= L/3 pieces has more than L/4, so a segment tree of Misra-Gries summaries
//   with three counters finds every candidate; each one's exact count comes from a binary
//   search in its sorted piece list. O((n + q) log n).
// original: corpus/submissions/4253124.cpp
// Bounds: n, m, q: 1..=2e5; flavour: 1..=m; asks.l, asks.r: 1..=n

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Piece = 1..=n
domain Flavour = 1..=m
let flavour: [Piece]Flavour = read()
let q: 1..=2e5 = read()
let asks: [q](l: Piece, r: Piece) = read()

// node v (leaves n..2n, node 0 a query's fold) keeps its counters in slots 3v..3v + 3 of
// (fl, ct); a count of 0 is an empty counter
var (fl, ct) = (List.filled(6 * n, 1), List.filled(6 * n, 0))
var (sf, sc) = (List.filled(6, 1), List.filled(6, 0))
fn join_into(d: Int, a: Int, b: Int):
    var k = 0
    for s in 3 * a..3 * a + 3:
        if ct[s] > 0:
            (sf[k], sc[k]) = (fl[s], ct[s])
            k += 1
    let from_a = k
    for s in 3 * b..3 * b + 3:
        if ct[s] > 0:
            var j = 0
            while j < from_a and sf[j] != fl[s]: j += 1
            if j < from_a: sc[j] += ct[s]
            else:
                (sf[k], sc[k]) = (fl[s], ct[s])
                k += 1
    for i in 1..k:  // largest count first
        var j = i
        while j > 0 and sc[j - 1] < sc[j]:
            (sf[j - 1], sf[j], sc[j - 1], sc[j]) = (sf[j], sf[j - 1], sc[j], sc[j - 1])
            j -= 1
    let cut = if k > 3: sc[3] else: 0
    for t in 0..3:
        let keep = t < k and sc[t] > cut
        (fl[3 * d + t], ct[3 * d + t]) = (if keep: sf[t] else: 1, if keep: sc[t] - cut else: 0)

for p in Piece: (fl[3 * (n + p.ord)], ct[3 * (n + p.ord)]) = (flavour[p].label, 1)
for v in (1..n).rev: join_into(v, 2 * v, 2 * v + 1)

let at = flavour.buckets()  // the pieces of each flavour, in order
fn below(f: Flavour, p: Piece) -> Int = (0..at[f].len).first_true(i => at[f][i] >= p) ?? at[f].len
fn above(f: Flavour, p: Piece) -> Int = (0..at[f].len).first_true(i => at[f][i] > p) ?? at[f].len

for (l, r) in asks:
    for t in 0..3: ct[t] = 0
    var (a, b) = (n + l.ord, n + r.ord + 1)
    while a < b:
        if a % 2 == 1:
            join_into(0, 0, a)
            a += 1
        if b % 2 == 1:
            b -= 1
            join_into(0, 0, b)
        (a, b) = (a / 2, b / 2)
    var (top, second) = (0, 0)
    for t in 0..3:
        if ct[t] > 0:
            let f = Flavour(fl[t])
            let c = above(f, r) - below(f, l)
            if c > top: (top, second) = (c, top)
            elif c > second: second = c
    let len = r.label - l.label + 1
    print(yes(3 * top >= 2 * len or 3 * second >= len))
