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
// Bounds: n: 2..=50; m: 2..=400; tube: 1..=n
// original: corpus/submissions/5445674.cpp
let (n, m) = read<(2..=50, 2..=400)>()
domain Tube = 1..=n + 1
var tube = [Tube] t => if t.label <= n: read<[m]1..=n>().collect() else: List()
var moves = List<(Tube, Tube)>()
var spare = Tube.last!

fn mv(from: Tube, to: Tube, count: Int = 1):
    for _ in 0..count:
        tube[to].push(tube[from].pop()!)
        moves.push((from, to))

fn split(mut given: List<Tube>, lo: Int, hi: Int):
    if lo == hi:
        return
    let mid = (lo + hi) / 2
    fn low(t: Tube) -> Int = tube[t].count(c => c <= mid)
    fn high_bottom(t: Tube) -> Bool = tube[t].first! > mid
    fn mixed(t: Tube) -> Bool = 0 < low(t) < m
    fn bottoms(t: Tube) -> Int = if high_bottom(t): m - low(t) else: low(t)

    // each tube: the larger half goes to the bottom, the other on top
    for a in given:
        let helper = Tube.find(t => t != a and t != spare)!
        let lows = low(a)
        mv(helper, spare, lows)
        for _ in 0..m:
            mv(a, if tube[a].last! > mid: spare else: helper)
        if lows > m - lows:
            mv(helper, a, lows); mv(spare, a, m - lows)
        else:
            mv(spare, a, m - lows); mv(helper, a, lows)
        mv(spare, helper, lows)

    // two mixed tubes with the same bottom half pour the tops off and fill one with bottoms
    while let (a, b) = given.pairs().find(p => mixed(p.0) and mixed(p.1) and high_bottom(p.0) == high_bottom(p.1)):
        let (a0, b0) = (bottoms(a), bottoms(b))
        mv(a, spare, m - a0); mv(b, spare, m - b0)
        mv(b, a, m - a0)
        let (left, poured) = (a0 + b0 - m, 2 * m - a0 - b0)
        if left > poured:
            mv(spare, b, poured)
        else:
            mv(b, spare, left)
            given = [if t == b: spare else: t for t in given]
            spare = b

    // at most one mixed pair is left, with different bottoms: trade their tops
    let rest = [t for t in given if mixed(t)]
    if rest.len > 0:
        let (a, b) = if high_bottom(rest[0]): (rest[1], rest[0]) else: (rest[0], rest[1])
        let highs = m - low(a)
        mv(a, spare, highs); mv(b, a, low(b)); mv(spare, b, highs)
    var lows = [t for t in given if not high_bottom(t)]
    var highs = [t for t in given if high_bottom(t)]
    split(mut lows, lo, mid)
    split(mut highs, mid + 1, hi)

var tubes = Tube.take(n).collect()
split(mut tubes, 1, n)
print(moves.len)
for (a, b) in moves:
    print(a, b)
