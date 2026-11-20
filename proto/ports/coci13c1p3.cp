// cp: 0.1 a73805c
// COCI 2013 Ratar: count pairs of rectangles that touch at exactly one corner
// point and have equal sums. For every grid point, compare the rectangles on
// opposite sides of it, one diagonal at a time.
let n: 1..=50 = read()
domain I = 1..=n
type B = I.bounds
let a: [I, I]-1000..=1000 = read()

var pre: [B, B]Int = 0
for r in I:
    for c in I:
        pre[r.r, c.r] = pre[r.r, c.l] + pre[r.l, c.r] - pre[r.l, c.l] + a[r, c]

// sum of the rectangle between two grid points
fn area(r1: B, c1: B, r2: B, c2: B) -> Int:
    pre[r2, c2] - pre[r1, c2] - pre[r2, c1] + pre[r1, c1]

// pairs (x, y) with x == y
fn matches(xs: List<Int>, ys: List<Int>) -> Int:
    var seen: HashMap<Int, Int> = []
    for x in xs: seen[x] += 1
    sum(seen.get(y) ?? 0 for y in ys)

var total = 0
for i in I:
    for j in I:
        // the grid point at the top-left corner of cell (i, j)
        let (r, c) = (i.l, j.l)
        let se = [area(r, c, x.r, y.r) for x in i..=I.last for y in j..=I.last]
        let nw = [area(x.l, y.l, r, c) for x in I.first..i for y in I.first..j]
        let sw = [area(r, y.l, x.r, c) for x in i..=I.last for y in I.first..j]
        let ne = [area(x.l, c, r, y.r) for x in I.first..i for y in j..=I.last]
        total += matches(se, nw) + matches(sw, ne)
print(total)
