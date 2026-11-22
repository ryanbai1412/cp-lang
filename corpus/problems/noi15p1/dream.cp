// NOI 2015 "Program Auto-Analysis": can x_i = x_j / x_i != x_j constraints
// over ids up to 1e9 all hold? Union the equalities, then test the rest.
for _ in 0..read<Int>():
    let cons = read<List<(i: 1..=1e9, j: 1..=1e9, same: Bool)>>()
    let (domain X, at) = compress(cons.flat_map(c => [c.i, c.j]))
    var eq = Dsu<X>()
    for c in cons.filter(.same):
        eq.unite(at[c.i], at[c.j])
    print(yes(cons.filter(c => not c.same).all(c => not eq.same(at[c.i], at[c.j]))), yes: "YES", no: "NO")
