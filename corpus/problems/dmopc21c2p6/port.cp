// Source: DMOPC '21 Contest 2 P6 (Strange Function), https://dmoj.ca/problem/dmopc21c2p6
// limits: 2000 ms, 256 MB
// Problem: a permutation p of 1..N and Q queries, N, Q <= 3e5. f(l, r) finds the minimum of
//   p[l..=r] at i, rotates p[l..=i] right by one so it lands at l, then calls f(i + 1, r).
//   Query 1: apply f(1, N). Query 2 i: print p_i. Query 3 i: print j with p_j = i.
// Solution: fix a threshold and call larger values big. After k passes the big positions are the
//   original ones shifted right by k, except a suffix packed against the end: position i is big
//   iff i - k is big or the bigs in [i - k, N) number at least N - i. All queries are offline
//   with their pass count k. Query 2: `parallel_bsearch` over the values, largest first, with a Fenwick
//   tree of big positions. Query 3: sweep thresholds down; Fenwicks of big count and index sum give the
//   sum of big positions after k passes, and x's position is that sum with x minus without.
//   O((N + Q) log^2 N).
// original: corpus/submissions/3968585.cpp
// Pass 1: Parallel binary search over values 0..n-1.
// Pass 2: Sweep values descending with two Fenwick trees (count and index sum).
use std.search.{parallel_bsearch}

let n: 1..=3e5 = read()
let q: 1..=3e5 = read()
domain Pos = 0..n  // from 0: a pass moves a big value from i - k to i
domain Val = 1..=n
domain Ask = 0..q
let p: [Pos]Val = read()
let place = p.inverse()

var value_asks: List<(at: Int, passes: Int, ask: Ask)> = []
var place_asks: [Val]List<(passes: Int, ask: Ask)> = []
var passes = 0
for ask in Ask:
    let kind: 1..=3 = read()
    if kind == 1:
        passes += 1
    elif kind == 2:
        let i: 1..=n = read()
        value_asks.push((i - 1, passes, ask))
    else:
        let v: Val = read()
        place_asks[v].push((passes, ask))

fn cut(c: Int) -> Cut<Pos> = Pos.cuts.at(max(c, 0))!
var answer: [Ask]?Int = None

// with the values from v up big, position i is big after k passes
var bigs = Fenwick<Pos, Int>()
fn is_big(i: Int, k: Int) -> Bool:
    (i >= k and bigs.get(Pos(i - k)) == 1) or bigs.suffix(cut(i - k)) >= n - i

let first_big = parallel_bsearch(
    value_asks.len, 0..n,
    reset: () => bigs.clear(),
    apply: t => bigs.add(place[Val(n - t)], 1),
    ok: j => is_big(value_asks[j].at, value_asks[j].passes),
)
for (a, t) in zip(value_asks, first_big): answer[a.ask] = n - t!

// the sum of the big positions after k passes: a packed suffix, the rest shifted by k
var count = Fenwick<Pos, Int>()
var place_sum = Fenwick<Pos, Int>()
fn big_place_sum(k: Int) -> Int:
    let packed_from = (0..=n).first_true(c => count.suffix(cut(c - k)) >= n - c)!
    let packed = n - packed_from
    (packed_from + n - 1) * packed / 2 + (count.all - packed) * k + place_sum.prefix(cut(packed_from - k))

fn toggle(v: Val, on: Int):
    count.add(place[v], on)
    place_sum.add(place[v], on * place[v].ord)

for v in Val.rev:
    toggle(v, 1)
    for a in place_asks[v]:
        toggle(v, -1)
        let without = big_place_sum(a.passes)
        toggle(v, 1)
        answer[a.ask] = big_place_sum(a.passes) - without + 1

for ask in Ask:
    if let x = answer[ask]: print(x)
