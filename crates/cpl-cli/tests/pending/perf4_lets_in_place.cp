let n: Int = read()
var rows: [0..n]List<Int> = []
var grid: List<List<Int>> = []
var words: List<Str> = []
for i in 0..n:
    rows[i].push(i)
    rows[i].push(i + 1)
    grid.push([i, 2 * i])
    words.push("w{i}")

fn bump(i: Int):
    rows[i].push(100)

fn size(i: Int) -> Int: rows[i].len

var total = 0
// Nothing after the `let` changes `rows`: no copy.
for i in 0..n:
    let row = rows[i]
    let word = words[i]
    total += row.len + size(i) + word.len
// Each of these is what the row was before the change.
for i in 0..n:
    let before = rows[i]
    bump(i)
    total += before.len * 1000
for i in 0..n:
    let kept = rows[i]
    rows[i].push(7)
    rows[i][0] = -1
    total += kept.len * 100000 + kept[0]
for i in 0..n:
    let g = grid[i]
    grid.push([1])
    grid[i][1] = 50
    total += g[1]
for i in 0..n:
    let was = rows[i]
    let f = k => rows[k].push(3)
    f(i)
    total += was.len
for i in 0..n:
    let old = rows[i]
    rows[i] = []
    total += old.len
    rows[i] = old

fn inside(i: Int) -> Int:
    if i > 0:
        let a = rows[i]
        return a.len + size(i)
    let b = rows[i]
    bump(i)
    b.len + rows[i].len

print(total, inside(0), inside(1), rows[0], grid.len)

@!multitest
let k: Int = read()
let pick = rows[k]
rows[k].push(k)
print(pick, rows[k].len)
