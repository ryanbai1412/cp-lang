// NOI 2008 "Candy Rain": clouds bounce between 0 and m; add and remove clouds
// and count clouds over [l, r] at time t. A cloud is a point (phase, len) on
// a 2m cycle; a query is two rectangles, one per direction of travel.
enum Op = Add(t: Int, c: Int, l: Int, r: Int, d: -1..=1) | Ask(t: Int, l: Int, r: Int) | Remove(c: Int)

let (q, m) = read<(1..=1e6, 1..=1000)>()
let cycle = 2 * m
domain X = [3 * cycle]
var rising = Fenwick2D<X, X, Int>()
var falling = Fenwick2D<X, X, Int>()
var cloud = Map<Int, (at: Int, len: Int)>()

fn put(at: Int, len: Int, v: Int):
    for shift in [0, cycle]:
        rising.add(X(at + shift), X(at + len + shift), v)
        falling.add(X(at + cycle + shift), X(at - len + cycle + shift), v)

fn span(lo: Int, hi: Int) -> Range<X> = X(lo)..=X(hi)

for op in read<[q]Op>():
    match op:
        Add(t, c, l, r, d) =>
            let at = ((if d == 1: l else: cycle - l) - t).mod(cycle)
            cloud[c] = (at: at, len: r - l)
            put(at, r - l, 1)
        Ask(t, l, r) =>
            let base = (-t).mod(cycle)
            let up = rising.sum(span(base, base + r), span(base + l, base + l + cycle - 1))
            let b = base + cycle
            let down = falling.sum(span(b + max(r + 1, cycle - r), b + cycle - 1), span(b - l + 1, b + cycle - l))
            print(up + down)
        Remove(c) =>
            let x = cloud.remove(c)!
            put(x.at, x.len, -1)
