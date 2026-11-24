// Source: DMPG '15 G6 (Yōkan), https://dmoj.ca/problem/dmpg15g6
// Problem: n <= 2e5 pieces with flavours in 1..=m, q <= 2e5 queries l r: can two people each
//   get >= L/3 pieces (L = r - l + 1) of one flavour, the same or two different? Print YES or
//   NO per query.
// Solution: YES iff the top count c1 >= 2L/3, or the top two counts are both >= L/3. A
//   flavour with >= L/3 pieces has more than L/4, so a segment tree of Misra-Gries summaries
//   with three counters finds every candidate; each one's exact count comes from a binary
//   search in its sorted piece list. O((n + q) log n).
// original: corpus/submissions/4253124.cpp

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Piece = 1..=n
domain Flavour = 1..=m
let flavour: [Piece]Flavour = read()
let q: 1..=2e5 = read()
let asks: [q](l: Piece, r: Piece) = read()

// a Misra-Gries summary with three counters: it keeps every flavour of more than a quarter
type Heavy = List<(flavour: Flavour, n: Int)>
fn join(a: Heavy, b: Heavy) -> Heavy:
    var all: Heavy = []
    for x in a + b:
        if let i = (0..all.len).find(i => all[i].flavour == x.flavour): all[i].n += x.n
        else: all.push(x)
    if all.len <= 3: return all
    all.sort(by: x => desc(x.n))
    let cut = all[3].n
    [(x.flavour, x.n - cut) for x in all[0..3] if x.n > cut]

let heavy = SegTree<Piece, Heavy>(Monoid([], join), [Piece] p => [(flavour[p], 1)])
let at = flavour.buckets()  // the pieces of each flavour, in order
fn below(f: Flavour, p: Piece) -> Int = (0..at[f].len).first_true(i => at[f][i] >= p) ?? at[f].len
fn above(f: Flavour, p: Piece) -> Int = (0..at[f].len).first_true(i => at[f][i] > p) ?? at[f].len

for (l, r) in asks:
    let len = r.label - l.label + 1
    var counts = [above(h.flavour, r) - below(h.flavour, l) for h in heavy.fold(l..=r)]
    counts.sort(by: c => desc(c))
    let top = counts.get(0) ?? 0
    let second = counts.get(1) ?? 0
    print(yes(3 * top >= 2 * len or 3 * second >= len))
