// COCI 2013 Ratar: count pairs of rectangles that touch at exactly one corner
// point and have equal sums. For every grid point, compare the rectangles on
// opposite sides of it, one diagonal at a time.
let n: 1..=50 = read()
domain I = 1..=n
let a: [I, I]-1000..=1000 = read()
let pre = prefix(a)

// pairs (x, y) with x == y
fn matches(xs: List<Int>, ys: List<Int>) -> Int:
    let seen = counts(xs)
    sum(seen[y] for y in ys)

var total = 0
for r in I.cuts:
    for c in I.cuts:
        // sums of the rectangles with one corner at grid point (r, c) and the
        // opposite corner cell in the given quadrant
        fn corner(rows: Range<I>, cols: Range<I>) -> List<Int>:
            [pre.sum(span(r, x), span(c, y)) for x in rows for y in cols]
        total += matches(corner(..r, ..c), corner(r.., c..)) + matches(corner(..r, c..), corner(r.., ..c))
print(total)
