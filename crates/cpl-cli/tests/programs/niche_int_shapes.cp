// `?Int` tables in every place one is made: a grid, a list grown by `push`,
// a table in a loop, in a fn, and per test of a `@multitest`. An index that
// runs code moves the table out of line, so that table is not packed.
@multitest
let n: 1..=6 = read()
var calls = 0
fn bump() -> Int:
    calls += 1
    calls % 3
var g: [n, n]?Int = None
var t: List<?Int> = []
for i in 0..n:
    t.push(i * 2)
    g[i, 0] = i
    if i % 2 == 0:
        var inner: [n]?Int = None
        inner[i] = i + 100
        t.push(inner[i])
        t.push(inner[0])
print(g[bump(), bump() - bump() + 0] ?? -1, t[bump()] ?? -1, yes(g[1 % n, 0] == None), calls, t.len)
var late: [n]?Int = None
if n > 2:
    late[2] = 7
print(late[n - 1] ?? -1, yes(late[0] == None))
fn work(k: Int) -> Int:
    var acc = 0
    for r in 0..k:
        var row: List<?Int> = []
        for j in 0..=r: row.push(if j % 2 == 0: None else: j)
        row[0] = r
        for j in 0..=r: acc += row[j] ?? 1
    acc
print(work(n))
