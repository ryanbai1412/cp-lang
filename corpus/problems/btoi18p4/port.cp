// Source: BOI 2018 P4 (Alternating Current), https://dmoj.ca/problem/btoi18p4
// limits: 1000 ms, 1024 MB
// Problem: a circle of n <= 1e5 segments and m <= 1e5 wires, each covering a cyclic range a..b.
//   Direct each wire (0 or 1) so every segment is covered by wires of both directions; print
//   the m digits, or "impossible".
// Solution: a wire inside another takes the opposite direction of its outermost holder, so only
//   the k outermost wires, in circle order, matter (sweep the doubled circle sorted by start,
//   longest first). k even or 1: alternate them; k = 3: try each rotation. Otherwise two
//   neighbours b, c share a direction: that works iff every other top is covered by its own
//   nested wires and its two neighbours, and b..c by the nested wires of b and c plus the tops
//   either side; try each pair, alternate from c and verify both directions cover the ring.
//   O(m log m).
// original: corpus/submissions/4656863.cpp
// Bounds: n: 3..=1e5; m: 2..=1e5; ends.a, ends.b: 1..=n
let (n, m) = read<(3..=1e5, 2..=1e5)>()
domain Seg = 1..=n
domain Wire = [m]
let ends = read<[Wire](a: Seg, b: Seg)>()
let lo = [Wire] w => Int(ends[w].a)
let hi = [Wire] w => Int(ends[w].b) + (if ends[w].b < ends[w].a: n else: 0)

// Whether the arcs `on`, each repeated one lap either way, cover segments lo..=hi.
// O(s log s) in the number of arcs, never O(n), so the per-top checks stay O(m log m) in total.
fn covers(on: List<Wire>, part: (lo: Int, hi: Int)) -> Bool:
    var upto = part.lo - 1
    for (l, h) in [(lo[w] + s, hi[w] + s) for w in on for s in [-n, 0, n]].sorted():
        if upto >= part.hi:
            break
        if l > upto + 1:
            return false
        upto max= h
    upto >= part.hi

let whole = (lo: 1, hi: n)
var holder: [Wire]?Wire = None
var reach: ?(end: Int, by: Wire) = None
let copies = [(start: lo[w] + s, end: hi[w] + s, w:) for w in Wire for s in [0, n]]
for c in copies.sorted(by: c => (c.start, desc(c.end))):
    match reach:
        Some(r) if c.end <= r.end and r.by != c.w => holder[c.w] = holder[c.w] ?? Some(r.by)
        _ => reach = Some((end: c.end, by: c.w))
let top = [Wire] w =>
    var u = w
    while let h = holder[u]: u = h
    u
let tops = Wire.filter(w => holder[w] == None).collect().sorted(by: w => lo[w])
domain T = [tops.len]
var rank: [Wire]?T = None
for t in T:
    rank[tops[Int(t)]] = Some(t)
let inside = Wire.filter(w => holder[w] != None).collect().group(by: w => top[w])
fn kids(t: T) -> List<Wire> = inside.get(tops[Int(t)]) ?? List()

fn spread(side: T -> Bool) -> ?[Wire]Bool:
    let dir = [Wire] w => if holder[w] == None: side(rank[w]!) else: not side(rank[top[w]]!)
    let both = [false, true].all(d => covers(Wire.filter(w => dir[w] == d).collect(), whole))
    if both: Some(dir) else: None

// Top t can share its neighbour's direction when its own nested wires and its two neighbouring
// tops still cover it from the other side.
fn alone(t: T) -> Bool =
    covers(
        kids(t) + [tops[Int(T.wrap(t.ord - 1))], tops[Int(T.wrap(t.ord + 1))]],
        (lo: lo[tops[Int(t)]], hi: hi[tops[Int(t)]]),
    )

let answer: ?[Wire]Bool =
    if T.count % 2 == 0 or T.count == 1:
        spread(t => Int(t) % 2 == 1)
    elif T.count == 3:
        T.first_some(b => spread(t => T.wrap(t.ord - b.ord).ord % 2 == 0))
    elif not covers(Wire.collect(), whole):
        None
    else:
        let free = [T] t => alone(t)
        let good = T.filter(t => free[t]).collect().len
        T.first_some(b =>
            let c = T.wrap(b.ord + 1)
            let (wb, wc) = (tops[Int(b)], tops[Int(c)])
            let pair = (lo: lo[wb], hi: hi[wc] + (if lo[wc] < lo[wb]: n else: 0))
            let others = [tops[Int(T.wrap(b.ord - 1))], tops[Int(T.wrap(c.ord + 1))]]
            if good - Int(free[b]) - Int(free[c]) == T.count - 2 and covers(
                kids(b) + kids(c) + others,
                pair,
            ):
                spread(t => T.wrap(t.ord - c.ord).ord % 2 == 0)
            else: None
        )

match answer:
    Some(dir) => print(dir.map(d => Int(d)), sep: "")
    None => print("impossible")
