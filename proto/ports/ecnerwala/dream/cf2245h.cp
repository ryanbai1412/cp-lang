// Codeforces 2245 H: Connect Connect See.
// A path with at most two turns is V-H-V or H-V-H, any leg possibly empty: u and v
// walk their columns to a row clear between them, or their rows to such a column.
// V-H-V, row by row: a column offers the cells whose column walk reaches row r (the
// cell itself if nonzero, else the nearest nonzero above and below), and a stretch of
// row r with only zeros inside joins everything it offers. Count each pair at the
// first row joining it: w meets an x present before it for the first time iff no row
// since w arrived let w's column see x's. H-V-H pairs are few, at most 4 per column
// for each pair of rows (n <= 100), so list them and add those V-H-V missed.
// O(n * nm) time, O(nm log n) memory.
@!multitest
let (n, m): (1..=100, 1..=2e6) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (row: Row, col: Col)
let a: [Cell]0..=2e6 = read()
let in_row = [Row] r => Walls([Col] c => a[r, c] > 0)
let in_col = [Col] c => Walls([Row] r => a[r, c] > 0)
fn rows_of(w: Cell) -> Range<Row> = in_col[w.col].run(w.row)
fn cols_of(w: Cell) -> Range<Col> = in_row[w.row].run(w.col)

let left_seen = [Col] c => SparseTable(min, [Row] r => in_row[r].view(c).start)
let right_seen = [Col] c => SparseTable(max, [Row] r => in_row[r].view(c).end)
fn seen(c: Col, rows: Range<Row>) -> Range<Col> =       // columns some row of `rows` shows c
    if rows.is_empty: c..c else: left_seen[c].fold(rows)..right_seen[c].fold(rows)
fn joined_by_row(u: Cell, v: Cell) -> Bool = v.col in seen(u.col, rows_of(u) & rows_of(v))

var pairs = 0
for r in Row, stretch in in_row[r].stretches:
    let here = [(row: q, col: c) for c in stretch for q in in_col[c].arrivals(r)]
    var met = Map<Int, SortedBag<Col>>()                 // columns of cells already here
    for w in here.sort(by: w => (rows_of(w).start, w)):
        let known = seen(w.col, rows_of(w).start..r)
        pairs += met[a[w]].len - met[a[w]].range(known).len
        met[a[w]].add(w.col)

for top in Row, bottom in Row.after(top):
    var last_open: ?Col = none
    for c in Col if bottom in in_col[c].view(top):
        for x in in_row[top].arrivals(c), y in in_row[bottom].arrivals(c):
            let (u, v) = ((row: top, col: x), (row: bottom, col: y))
            let again = last_open is some(l) and l in (cols_of(u) & cols_of(v))
            if a[u] == a[v] and not again and not joined_by_row(u, v): pairs += 1
        last_open = c
print(pairs)
