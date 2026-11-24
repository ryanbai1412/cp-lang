let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
var g: [Row, Col]Char = read()

// g.rows[r] lends one row as a [Col]Char: any fn written for a one-dimensional array takes it
fn count_hash(xs: [Col]Char) -> Int:
    count(x == '#' for x in xs)
print([count_hash(g.rows[r]) for r in Row])

// two rows at once need `r1 != r2`, the law of a row view
let r1 = Row.first!
let r2 = Row.last!
if r1 != r2:
    swap(mut g.rows[r1], mut g.rows[r2])
g.rows[r1].sort()
for r in Row:
    let row = g.rows[r]
    print([c for c in row])

// g[p] with p: (Row, Col) is the same element as g[r, c]: a[i, j] is sugar for a[(i, j)]
type Cell = (Row, Col)
let p: Cell = (r2, Col.first!)
g[p] = '@'
print(g[r2, Col.first!], yes(g[p] == g[r2, Col.first!]))

// distinct elements of one array, proven by a fact
fn bump2(mut x: Int, mut y: Int):
    x += 1
    y += 10
var cnt: [Col]Int = 0
let c1 = Col.first!
let c2 = Col.last!
if c1 != c2:
    bump2(mut cnt[c1], mut cnt[c2])
    swap(mut cnt[c1], mut cnt[c2])
print(cnt)
