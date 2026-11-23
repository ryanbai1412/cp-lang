// CF 1985H1 Maximize the Largest Component (Easy): fill one row or column with '#'.
// Filling row r joins every component touching rows r-1, r, r+1 and adds the '.' cells of r.
// Columns are rows of the transposed grid, so one fn answers both.

fn best_line<A: Domain, B: Domain>(g: [A, B]Char) -> Int:
    var d = Dsu<(A, B)>()
    for p in (A, B) if g[p] == '#':
        for q in p.adj4 if g[q] == '#':
            d.unite(p, q)
    var seen: [(A, B)]?A = None        // root -> the last line that counted it
    var best = 0
    for r in A:
        var total = B.count(c => g[r, c] == '.')
        for dr in -1..=1:
            let Some(rr) = A.at(Int(r) + dr) else: continue
            for c in B if g[rr, c] == '#':
                let root = d.find((rr, c))
                if seen[root] != Some(r):
                    seen[root] = Some(r)
                    total += d.size(root)
        best max= total
    best

@!multitest
let (n, m): (1..=1e6, 1..=1e6) = read()
domain R = 0..n
domain C = 0..m
let g = read<[R, C]Char>()
print(max(best_line(g), best_line(g.transposed())))
