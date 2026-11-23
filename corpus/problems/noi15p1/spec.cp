// noi15p1 Program Analyzer: are the constraints x_i = x_j and x_i != x_j satisfiable together? Join
// every equality, then no inequality may fall inside one set. Variable ids go up to 1e9.
@!multitest
let n = read<1..=1_000_000>()
let cons = read<[n](i: 1..=1_000_000_000, j: 1..=1_000_000_000, eq: 0..=1)>()
let (domain X, ids) = compress(chain(cons.map(c => c.i), cons.map(c => c.j)))
var dsu = Dsu<X>()
for c in cons.filter(c => c.eq == 1):
    dsu.unite(ids.index_of(c.i), ids.index_of(c.j))
print(yes(cons.all(c => c.eq == 1 or not dsu.same(ids.index_of(c.i), ids.index_of(c.j)))))
