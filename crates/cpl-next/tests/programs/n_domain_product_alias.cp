// expect: ok, output "4 4\n5 2\n4\n6"
// rule: R04.2.10 R04.2.11
// `type Cell = (Row, Col)` is the product domain; `Cell((r, c))` converts each part; a type
// parameter in a product key (`[V, V]T`) is a domain.
domain Row = 1..=2
domain Col = 1..=3
type Cell = (Row, Col)
var grid: [Cell]Char = '.'
grid[Row(1), Col(2)] = '#'
grid[Cell((2, 3))] = '#'
var n = 0
for p in Cell:
    if grid[p] == '.':
        n += 1
print(n, Cell.count - 2)
let p = Cell((2, 3))
print(p.ord, Int(p._0))
fn diag<V>(t: [V, V]Int) -> Int:
    var s = 0
    for v in V:
        s += t[v, v]
    return s
let id = [Row, Row] (a, b) => if a == b: 2 else: 0
print(diag(id))
let w = [Col, Col] (a, b) => 2
print(diag(w))
