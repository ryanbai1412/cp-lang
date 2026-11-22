// cco18p4 (interactive): the grid's values are unimodal along rows and columns. Ternary search the
// first column for the best row, then that row for the best column. ternary_min is golden-section,
// and ask is memoized, so the probes fit the query budget.
let wire = Wire()
let (n, m, k) = read<(1..=1_000_000, 1..=1_000_000, Int)>()
@memo(map)
fn ask(x: Int, y: Int) -> Int:
    print("?", x, y)
    read()
let x = ternary_min(1..=n, i => ask(i, 1))!
let y = ternary_min(1..=m, j => ask(x, j))!
print("!", ask(x, y))
