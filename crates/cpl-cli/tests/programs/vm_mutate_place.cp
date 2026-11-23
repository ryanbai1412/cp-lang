// Mutations the VM runs itself on a place it walks to: a local, a global,
// an element; as a statement and for their answer.
let n: Int = read()
var word = "ab"
var rows: List<List<Int>> = [[1], [2, 3]]
var seen: Set<Int> = Set()
fn grow(mut xs: List<Int>, k: Int):
    xs.extend([k, k])
    xs.swap(0, xs.len - 1)
for i in 0..n:
    word.push('c')
    rows[i % 2].reserve(4)
    rows[i % 2].extend([i, i * i])
    rows[1].push(i)
    var mine = [i, i + 1, i + 2]
    mine.swap(0, 2)
    mine.extend(rows[0])
    grow(mut mine, i)
    print(mine)
    if seen.insert(i % 3): print("new", i % 3)
    print(yes(seen.remove(i % 2)), seen.len)
var grid: [0..3]Int = 0
grid.fill(n)
grid.swap(0, 1)
print(word, rows, grid)
rows[0].truncate(2)
rows[0].resize(4, 9)
rows[1].clear()
word.clear()
print(word.len, rows)
