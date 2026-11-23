// A lambda of several parameters takes the fields of one tuple or record
// element (R01.8.1), the same way in every call that passes it one.
let cells: [3](r: Int, c: Int, v: Int) = [(r: 1, c: 1, v: 5), (r: 1, c: 2, v: 0), (r: 2, c: 2, v: 7)]
print(yes(cells.all((r, c, v) => r <= c)), yes(cells.any((r, c, v) => v == 0)))
print(cells.count((r, c, v) => r == c), cells.map((r, c, v) => r + c + v))
let pairs = [(1, 2), (3, 3)]
print(yes(pairs.all((x, y) => x <= y)), pairs.count((x, y) => x == y))
