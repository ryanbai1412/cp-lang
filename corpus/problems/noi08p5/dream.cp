// NOI 2008 "Candy Rain": clouds bounce between 0 and m; add and remove clouds
// and count clouds over [l, r] at time t. A cloud is a point (phase, len) on
// a 2m cycle; a query is two rectangles.
let (q, m) = read<(1..=200_000, 1..=1000)>()
domain Colour = 1..=1_000_000
enum Op:
    @tag(1) Add(t: 0..=2_000_000_000, c: Colour, l: 0..=m, r: 0..=m, d: -1..=1)
    @tag(2) Ask(t: 0..=2_000_000_000, l: 0..=m, r: 0..=m)
    @tag(3) Remove(t: 0..=2_000_000_000, c: Colour)

let cycle = 2 * m
domain X = 0..cycle
domain Y = 0..cycle + m
// clouds as points (phase, phase + len) rising and (phase, phase - len + m) falling; a query's
// rectangle lives on the doubled cycle, so it meets each point and the point's copy one cycle on
var rising = Fenwick2D<X, Y, Int>()
var falling = Fenwick2D<X, Y, Int>()
var cloud = Map<Colour, (at: Int, len: Int)>()

fn put(at: Int, len: Int, v: Int):
    rising.add(X(at), Y(at + len), v)
    falling.add(X(at), Y(at - len + m), v)

fn count(f: Fenwick2D<X, Y, Int>, xs: Range<Int>, ys: Range<Int>) -> Int:
    var total = 0
    for s in [0, cycle]:
        let a = (xs.start - s..xs.end - s).clip(0..cycle)
        let b = (ys.start - s..ys.end - s).clip(0..cycle + m)
        if not a.is_empty and not b.is_empty:
            total += f.fold(X(a.start)..=X(a.last!), Y(b.start)..=Y(b.last!))
    total

for op in read<[q]Op>():
    match op:
        Add(t, c, l, r, d) =>
            let at = ((if d == 1: l else: cycle - l) - t) % cycle
            cloud[c] = (at: at, len: r - l)
            put(at, r - l, 1)
        Ask(t, l, r) =>
            let base = -t % cycle
            let up = count(rising, base..=base + r, base + l..=base + l + cycle - 1)
            let down = count(falling, base + max(r + 1, cycle - r)..=base + cycle - 1, base - l + 1 + m..=base + cycle - l + m)
            print(up + down)
        Remove(_, c) =>
            let x = cloud.remove(c)!
            put(x.at, x.len, -1)
