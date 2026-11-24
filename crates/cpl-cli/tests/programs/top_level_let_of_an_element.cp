// A top-level `let` of a list element is a reference in the C++; later
// reads of the outer list leave it right.
let rows: List<List<Int>> = [[1, 2], [3, 4], [5]]
let r = rows[1]
print(r.sum(), r.len)
let first = rows[0]
print(first[1], rows.len)
print(rows[2].sum())
