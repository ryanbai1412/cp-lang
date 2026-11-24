// Rows and columns of a grid are places: written, sorted, reversed and
// swapped through, in a fn and at top level.
domain R = 0..3
domain C = 0..3
fn grid() -> [R, C]Int:
    var g: [R, C]Int = [R, C] (r, c) => 3 * Int(r) + 2 - Int(c)
    g.rows[R(0)].sort()
    reverse(mut g.cols[C(2)])
    let (i, j) = (R(1), R(2))
    if i != j:
        swap(mut g.rows[i], mut g.rows[j])
    g
var h = grid()
h.cols[C(0)].sort()
for r in R: print(h.rows[r])
fn line(mut g: [R, C]Int) -> Int:
    var row = g.rows[R(1)]
    row[C(0)] = 40
    var sum = 0
    for v in row: sum += v
    print(row)
    sum
print(line(mut h))
for r in R: print(h.rows[r])
var k = grid()
var col = k[.., C(1)]
col.reverse()
for v in col: print(v)
print(k.rows[R(0)])
