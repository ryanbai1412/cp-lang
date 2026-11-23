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
let lo = [Wire] w => Int(ends[w].a)
let hi = [Wire] w => Int(ends[w].b) + (if ends[w].b < ends[w].a: n else: 0)

// Whether the arcs `on`, each repeated one lap either way, cover segments lo..=hi.
// O(s log s) in the number of arcs, never O(n), so the per-top checks stay O(m log m) in total.
fn covers(on: List<Wire>, part: (lo: Int, hi: Int)) -> Bool:
    var reach = part.lo - 1
    for (l, h) in [(lo[w] + s, hi[w] + s) for w in on for s in [-n, 0, n]].sorted():
        if reach >= part.hi:
            break
        if l > reach + 1:
            return false
        reach = max(reach, h)
    reach >= part.hi

let whole = (lo: 1, hi: n)
var holder: [Wire]?Wire = []
var reach: ?(end: Int, by: Wire) = None
let copies = [(start: lo[w] + s, end: hi[w] + s, w: w) for w in Wire for s in [0, n]]
for c in copies.sorted(by: c => (c.start, desc(c.end))):
    match reach:
        Some(r) if c.end <= r.end and r.by != c.w => holder[c.w] = holder[c.w] ?? Some(r.by)
        _ => reach = Some((end: c.end, by: c.w))
fn top(w: Wire) -> Wire = iterate(from: Some(w), step: v => holder[v]).last()!
let tops = Wire.filter(w => holder[w].is_none()).collect().sorted(by: w => lo[w])
domain T = [tops.len]
var rank: [Wire]?T = []
for t in T:
    rank[tops[Int(t)]] = Some(t)
let inside = Wire.filter(w => holder[w].is_some()).collect().group(by: w => top(w))
fn kids(t: T) -> List<Wire> = inside.get(tops[Int(t)]) ?? List()

fn spread(side: T -> Bool) -> ?[Wire]Bool:
    let dir = [Wire] w => if holder[w].is_none(): side(rank[w]!) else: not side(rank[top(w)]!)
    let both = [false, true].all(d => covers(Wire.filter(w => dir[w] == d).collect(), whole))
    if both: Some(dir) else: None

// Top t can share its neighbour's direction when its own nested wires and its two neighbouring
// tops still cover it from the other side.
fn alone(t: T) -> Bool =
    covers(
        kids(t) + [tops[Int(T.ring.prev(t))], tops[Int(T.ring.next(t))]],
        (lo: lo[tops[Int(t)]], hi: hi[tops[Int(t)]]),
    )

let answer: ?[Wire]Bool =
    if T.len % 2 == 0 or T.len == 1:
        spread(t => Int(t) % 2 == 1)
    elif T.len == 3:
        T.first_some(b => spread(t => T.ring.dist(b, t) % 2 == 0))
    elif not covers(Wire.collect(), whole):
        None
    else:
        let free = [T] t => alone(t)
        let good = T.filter(t => free[t]).collect().len
        T.first_some(b =>
            let c = T.ring.next(b)
            let (wb, wc) = (tops[Int(b)], tops[Int(c)])
            let pair = (lo: lo[wb], hi: hi[wc] + (if lo[wc] < lo[wb]: n else: 0))
            let others = [tops[Int(T.ring.prev(b))], tops[Int(T.ring.next(c))]]
            if good - Int(free[b]) - Int(free[c]) == T.len - 2 and covers(
                kids(b) + kids(c) + others,
                pair,
            ):
                spread(t => T.ring.dist(c, t) % 2 == 0)
            else: None
        )

match answer:
    Some(dir) => print(dir.map(d => Int(d)), sep: "")
    None => print("impossible")
