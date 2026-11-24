// NOIP 2020 "Ball Moving": n + 1 tubes of capacity m, n colours of m balls,
// one tube empty; sort by colour in at most 820000 moves. Divide and conquer
// on colours lo..=hi: call colours up to mid "low", the rest "high"; sort
// each tube into a low and a high block (majority at the bottom), merge
// mixed tubes with the same bottom kind until at most one pair is left, fix
// that pair, then recurse on the tubes by bottom kind. O(n m log n) moves.
// copies: 1
let (n, m) = read<(2..=50, 2..=400)>()
domain Tube = 1..=n + 1
let start = read<[n][m]1..=n>()
var tube = [Tube] t => if Int(t) <= n: List(start[Int(t) - 1]) else: List()
var empty = Tube(n + 1)
var moves = List<(Tube, Tube)>()

fn mv(from: Tube, to: Tube, count: Int = 1):
    for _ in 0..count:
        tube[to].push(tube[from].pop()!)
        moves.push((from, to))

fn split(group: List<Tube>, lo: Int, hi: Int):
    if lo == hi:
        return
    let mid = (lo + hi) / 2
    fn low(t: Tube) -> Int = tube[t].count(c => c <= mid)
    fn bottom_low(t: Tube) -> Bool = tube[t].first! <= mid
    fn mixed(t: Tube) -> Bool = 0 < low(t) < m
    fn bottom_kind(t: Tube) -> Int = if bottom_low(t): low(t) else: m - low(t)

    // Sort each tube: lows go to a full helper, highs to the empty tube, then
    // the larger block goes back first.
    for a in group:
        let helper = Tube.find(t => t != a and t != empty)!
        let lows = low(a)
        mv(helper, empty, lows)
        for c in tube[a].rev.clone():
            mv(a, if c <= mid: helper else: empty)
        if lows > m - lows:
            mv(helper, a, lows); mv(empty, a, m - lows)
        else:
            mv(empty, a, m - lows); mv(helper, a, lows)
        mv(empty, helper, lows)

    // Merge two mixed tubes with the same bottom kind: one fills up with it,
    // and the leftovers (majority at the bottom again) stay in b or in the
    // empty tube, which then takes b's place in the group.
    var cols = group
    fn mergeable(a: Tube, b: Tube) -> Bool =
        a != b and mixed(a) and mixed(b) and bottom_low(a) == bottom_low(b)
    var merged = true
    while merged:
        merged = false
        for i in cols.keys:
            let a = cols[i]
            let j = cols.keys.find(j => mergeable(a, cols[j])) else: continue
            let b = cols[j]
            let (pa, pb) = (bottom_kind(a), bottom_kind(b))
            mv(a, empty, m - pa); mv(b, empty, m - pb)
            mv(b, a, m - pa)
            let (left, other) = (pa + pb - m, 2 * m - pa - pb)
            if left > other:
                mv(empty, b, other)
            else:
                mv(b, empty, left)
                swap(mut cols[j], mut empty)
            merged = true

    // At most two mixed tubes are left, one of each bottom kind: swap their tops.
    let rest = cols.filter(mixed).collect()
    if rest.len == 2:
        let (a, b) = if bottom_low(rest[0]): (rest[0], rest[1]) else: (rest[1], rest[0])
        let highs = m - low(a)
        mv(a, empty, highs); mv(b, a, low(b)); mv(empty, b, highs)

    let (lows, highs) = cols.partition(bottom_low)
    split(lows, lo, mid)
    split(highs, mid + 1, hi)

split(Tube.take(n).collect(), 1, n)
print(moves.len)
for (a, b) in moves:
    print(a, b)
