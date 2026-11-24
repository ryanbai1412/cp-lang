// Source: An Animal Contest 4 P4 (Reindeer Racing), https://dmoj.ca/problem/aac4p4
// limits: 1000 ms, 256 MB
// Problem: n lanes, m hurdles (n, m <= 2000); the baton starts in lane s, and when its holder hits
//   hurdle j it passes to lane k_j. A banana at (lane i, hurdle j) makes lane i hit hurdle j.
//   Given 0 <= L <= R <= nm, print a count X in L..=R and an n x m 0/1 matrix of X bananas
//   leaving the baton in lane f, or -1 if impossible.
// Solution: with s = f and L = 0 nothing is needed. Otherwise use j, the last hurdle passing to
//   f (the latest leaves the most free cells after it). Without one the baton must never
//   move: need s = f, and any lane but s may take bananas. If L <= n, a banana at (s, j) hands
//   the baton to f, and other lanes of column j are harmless. Otherwise fill column j (whoever
//   holds the baton passes it to f) and any cell except lane f after j. A plan works when
//   max(L, forced) <= min(R, forced + optional). O(nm).
// original: corpus/submissions/4140376.cpp
// Bounds: n, m: 1..=2000; s, f, pass: 1..=n; l: 0..=n * m; r: l..=n * m
let (n, m): (1..=2000, 1..=2000) = read()
domain Lane = 1..=n
domain Hurdle = 1..=m
let (s, f): (Lane, Lane) = read()
let l: 0..=n * m = read()
let r: l..=n * m = read()
let pass: [Hurdle]Lane = read()

type Cell = (lane: Lane, at: Hurdle)
let trivial = s == f and l == 0
let last_f = Hurdle.rev.find(h => pass[h] == f)

fn must() -> yields Cell:
    if trivial: return
    if let j = last_f:
        if l <= n: yield (s, j)
        else:
            for i in Lane: yield (i, j)

fn may() -> yields Cell:
    if trivial: return
    if let j = last_f:
        if l <= n:
            for i in Lane if i != s: yield (i, j)
        else:
            for i in Lane:
                for h in Hurdle if h != j and not (i == f and h > j): yield (i, h)
    elif s == f:
        for i in Lane if i != s:
            for h in Hurdle: yield (i, h)

let (sure, open) = (must().count(), may().count())
if (trivial or last_f != None or s == f) and max(l, sure) <= min(r, sure + open):
    let extra = max(0, l - sure)
    var banana: [Lane, Hurdle]0..=1 = 0
    for c in must(): banana[c.lane, c.at] = 1
    for c in may().take(extra): banana[c.lane, c.at] = 1
    print(sure + extra)
    for i in Lane: print([banana[i, h] for h in Hurdle])
else:
    print(-1)
