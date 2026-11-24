// NOI 2015 "Program Auto-Analysis": can x_i = x_j / x_i != x_j constraints
// over ids up to 1e9 all hold? Union the equalities, then test the rest.
@multitest
let q: Int = read()
let cons = read<[q](i: Int, j: Int, e: Int)>()
let (domain X, ids) = compress(cons.flat_map(c => [c.i, c.j]))
let at = (v: Int) => ids.index_of(v)
var eq = Dsu<X>()
for c in cons if c.e == 1: eq.unite(at(c.i), at(c.j))
print(yes(cons.filter(c => c.e != 1).all(c => not eq.same(at(c.i), at(c.j))), answers: ("YES", "NO")))
