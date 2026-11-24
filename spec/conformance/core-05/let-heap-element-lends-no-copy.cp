// expect: ok
// gap: compiler 2026-11-24T05Z
// rule: 05.3.5
// tool: build -o -
// lacks: "std::vector<ll> r;"
// `let r = rows[1]` lends the heap element; the C++ must not copy it into a fresh vector (acceptance run 6, row 4).
let rows: List<List<Int>> = [[1, 2], [3, 4]]
let r = rows[1]
print(r.sum())
