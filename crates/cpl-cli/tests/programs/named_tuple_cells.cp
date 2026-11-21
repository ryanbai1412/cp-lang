// Plain tuples fit named fields of the same types in order, held the same
// way: in arrays, optionals and lists too, without a copy.
let n = 2
var es: [n](Int, Int) = (0, 0)
es[1] = (3, 4)
let named: [n](a: Int, b: Int) = es
print(named[1].a + named[1].b)
let o: ?(Int, Int) = (5, 6)
let on: ?(a: Int, b: Int) = o
print(on!.b)
fn total(xs: [n](a: Int, b: Int)) -> Int = xs[1].a * xs[1].b
print(total(es))
let l: List<(Int, Int)> = [(1, 2)]
let ln: List<(x: Int, y: Int)> = l
print(ln[0].y)
