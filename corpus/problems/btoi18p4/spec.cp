// BOI 2018 "Alternating Current": wires cover wrapping arcs of a ring of n
// segments; give each wire a direction so both directions cover the ring.
// A wire inside another takes the opposite direction of its outermost holder.
// The k outermost wires alternate round the ring; when k is odd, two adjacent
// outermost wires share a direction, which works when the opposite side still
// covers their union from their own nested wires and their two neighbours.
let (n, m) = read<(3..=1e5, 2..=1e5)>()
domain Seg = 1..=n
domain Wire = [m]
let ends = read<[Wire](a: Seg, b: Seg)>()
domain Line = 1..=2 * n
let lo = [Wire] w => Line(Int(ends[w].a))
let hi = [Wire] w => Line(Int(ends[w].b) + (if ends[w].b < ends[w].a: n else: 0))

fn covers(on: List<Wire>, part: Range<Line>) -> Bool:
    var depth = [Line] _ => 0
    for w in on:
        for shift in [0, n]:
            depth.apply(Line.at(Int(lo[w]) + shift).., 1)
            depth.apply(Line.at(Int(hi[w]) + shift + 1).., -1)
    let run = depth.scan(+)
    part.all(p => run[p] + (Line.at(Int(p) + n).map(q => run[q]) ?? 0) > 0)

let whole = Line(1)..=Line(n)
var holder: [Wire]?Wire = []
var reach: ?(end: Int, by: Wire) = none
let copies = [(start: Int(lo[w]) + s, end: Int(hi[w]) + s, w: w) for w in Wire for s in [0, n]]
for c in copies.sorted(by: c => (c.start, desc(c.end))):
    match reach:
        some(r) if c.end <= r.end and r.by != c.w => holder[c.w] = holder[c.w] ?? some(r.by)
        _ => reach = some((end: c.end, by: c.w))
fn top(w: Wire) -> Wire = iterate(from: some(w), step: v => holder[v]).last()!
let tops = Wire.filter(w => holder[w].is_none()).collect().sorted(by: w => lo[w])
domain T = [tops.len]
var rank: [Wire]?T = []
for t in T:
    rank[tops[Int(t)]] = some(t)
let inside = Wire.filter(w => holder[w].is_some()).collect().group(by: w => top(w))

fn spread(side: T -> Bool) -> ?[Wire]Bool:
    let dir = [Wire] w => if holder[w].is_none(): side(rank[w]!) else: not side(rank[top(w)]!)
    let both = [false, true].all(d => covers(Wire.filter(w => dir[w] == d).collect(), whole))
    if both: some(dir) else: none

let answer: ?[Wire]Bool =
    if T.len % 2 == 0:
        spread(t => Int(t) % 2 == 1)
    else:
        T.first_some(b =>
            let c = T.ring.next(b)
            let (a, d) = (T.ring.prev(b), T.ring.next(c))
            let (wb, wc) = (tops[Int(b)], tops[Int(c)])
            let opposite = (inside.get(wb) ?? List()) + (inside.get(wc) ?? List()) + [tops[Int(a)], tops[Int(d)]]
            if covers(opposite, lo[wb]..=hi[wc]):
                spread(t => (T.ring.dist(b, t) % 2 == 0) != (t == c))
            else: none)

match answer:
    some(dir) => print(dir.map(d => d.int), sep: "")
    none => print("impossible")
