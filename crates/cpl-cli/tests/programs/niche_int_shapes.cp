// `?Int` tables in every place one is made: a grid, a list grown by `push`,
// a table in a loop, in a fn, and per test of a `@multitest`. An index that
// runs code moves the table out of line, so that table is not packed.
@multitest
let n: 1..=6 = read()
var calls = 0
fn bump() -> Int:
    calls += 1
    calls % 3
var g: [n, n]?Int = none
var t: List<?Int> = []
for i in 0..n:
    t.push(i * 2)
    g[i, 0] = i
    if i % 2 == 0:
        var inner: [n]?Int = none
        inner[i] = i + 100
        t.push(inner[i])
        t.push(inner[0])
print(g[bump(), bump() - bump() + 0] ?? -1, t[bump()] ?? -1, g[1 % n, 0] == none, calls, t.len)
var late: [n]?Int = none
if n > 2:
    late[2] = 7
print(late[n - 1] ?? -1, late[0] == none)
fn work(k: Int) -> Int:
    var acc = 0
    for r in 0..k:
        var row: List<?Int> = []
        for j in 0..=r: row.push(if j % 2 == 0: none else: j)
        row[0] = r
        for j in 0..=r: acc += row[j] ?? 1
    acc
print(work(n))
