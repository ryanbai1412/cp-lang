// NOIP 2020 "Ball Moving": n + 1 tubes of capacity m, n colours of m balls,
// one tube empty; sort by colour in at most 820000 moves. Divide and conquer
// on colours: split the tubes into "low" and "high" halves, then recurse.
let (n, m) = read<(Int, Int)>()
domain Tube = 1..=n + 1
var tube = [Tube] t => if t.label <= n: read<[m]Int>().collect() else: List()
var moves = List<(Tube, Tube)>()
var spare = Tube.last!

fn mv(from: Tube, to: Tube, count: Int = 1):
    for _ in 0..count:
        tube[to].push(tube[from].pop()!)
        moves.push((from, to))

fn split(given: List<Tube>, lo: Int, hi: Int):
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
        for c in tube[a].rev.clone():
            mv(a, if c > mid: spare else: helper)
        if lows > m - lows:
            mv(helper, a, lows); mv(spare, a, m - lows)
        else:
            mv(spare, a, m - lows); mv(helper, a, lows)
        mv(spare, helper, lows)

    // two mixed tubes with the same bottom half pour the tops off and fill one with bottoms
    var cols = given
    while let (a, b) = cols.pairs().find(p => mixed(p.0) and mixed(p.1) and high_bottom(p.0) == high_bottom(p.1)):
        let (a0, b0) = (bottoms(a), bottoms(b))
        mv(a, spare, m - a0); mv(b, spare, m - b0)
        mv(b, a, m - a0)
        let (left, poured) = (a0 + b0 - m, 2 * m - a0 - b0)
        if left > poured:
            mv(spare, b, poured)
        else:
            mv(b, spare, left)
            cols = [if t == b: spare else: t for t in cols]
            spare = b

    // at most one mixed pair is left, with different bottoms: trade their tops
    let rest = [t for t in cols if mixed(t)]
    if rest.len > 0:
        let (a, b) = if high_bottom(rest[0]): (rest[1], rest[0]) else: (rest[0], rest[1])
        let highs = m - low(a)
        mv(a, spare, highs); mv(b, a, low(b)); mv(spare, b, highs)
    let (lows, highs) = cols.partition(t => not high_bottom(t))
    split(lows, lo, mid)
    split(highs, mid + 1, hi)

split(Tube.take(n).collect(), 1, n)
print(moves.len)
for (a, b) in moves:
    print(a, b)
