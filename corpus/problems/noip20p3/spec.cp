// NOIP 2020 "Ball Moving": n + 1 tubes of capacity m, n colours of m balls,
// one tube empty; sort by colour in at most 820000 moves. Divide and conquer
// on colours: split the tubes into "low" and "high" halves, then recurse.
let (n, m) = read<(2..=50, 2..=400)>()
domain Tube = 1..=n + 1
var tube = [Tube] t => if Int(t) <= n: List(read<[m]1..=n>()) else: List()
var moves = List<(Tube, Tube)>()

fn mv(from: Tube, to: Tube, count: Int = 1):
    for _ in 0..count:
        tube[to].push(tube[from].pop()!)
        moves.push((from, to))

fn split(cols: List<Tube>, colours: Range<1..=n>):
    if colours.len <= 1:
        return
    let mid = colours.mid.l
    let spare = Tube.find(t => not cols.any(c => c == t) and tube[t].is_empty)!
    fn low(t: Tube) -> Int = tube[t].count(c => c <= mid)

    for a in cols:
        let helper = cols.find(t => t != a)!
        let (lo, hi) = (low(a), m - low(a))
        mv(helper, spare, m - lo)
        for c in tube[a].rev.clone():
            mv(a, if c <= mid: spare else: helper)
        if m - lo > lo:
            mv(helper, a, m - lo); mv(spare, a, lo)
        else:
            mv(spare, a, lo); mv(helper, a, m - lo)
        mv(spare, helper, m - lo)

    fn bottom_low(t: Tube) -> Bool = tube[t].first! <= mid
    fn mixed(t: Tube) -> Bool = 0 < low(t) < m
    while let (a, b) = cols.adjacent.find(
        p => mixed(p.0) and mixed(p.1) and bottom_low(p.0) == bottom_low(p.1)
    ):
        let side = if bottom_low(a): (t: Tube) => m - low(t) else: low
        mv(a, spare, side(a)); mv(b, spare, side(b))
        mv(b, a, m - tube[a].len)
    let (lo, hi) = cols.partition(bottom_low)
    split(lo, colours.l..mid.r)
    split(hi, mid.r..colours.r)

split(Tube.take(n).collect(), 1..=n)
print(moves.len)
for (a, b) in moves:
    print(a, b)
