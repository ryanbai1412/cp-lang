// Batch #16: `.items` on a `Str`, a `Map`, a `Deque` and a grid; a list of rows for a 2-D array.
let n: Int = read()
let s: Str = read()

// A `Str`: each character with its index.
for (i, c) in s.items: print(i, c)
let kept = s.items
print(kept.len, kept[1].1)
print([c for (i, c) in s.items if i != 1])

// A `Map`: each key with its value, smallest key first.
var m = Map<Int, Str>()
m[3] = "three"
m[1] = "one"
for (k, v) in m.items: print(k, v)
let pairs = m.items
print(pairs.len, pairs[0].0, pairs[1].1)
var h = HashMap<Int, Int>()
h[7] = 9
print(sum(k * v for (k, v) in h.items))

// A `Deque`: each element with its place from the front.
var dq = Deque<Int>()
dq.push(5)
dq.push(6)
dq.push_front(4)
for (i, x) in dq.items: print(i, x)
print([x * i for (i, x) in dq.items])

// A list of rows gives a 2-D array its cells.
let g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]
let b: [1..=2, 1..=3]Int = [[1, 2, 3], [4, 5, 6]]
print(g[1, 2], b[2, 3], g[0, ..], b[1.., 2])
let d: [0..n, 1..=n]Str = [["a", "b"], ["c", "d"]]
print(d[1, 1], d[0, 2])
let t: [0..2, 0..1, 0..2]Int = [[[1, 2]], [[3, 4]]]
print(t[1, 0, 1])

// A grid: each cell with the tuple of its indices, row by row.
for ((r, c), x) in b.items: print(r, c, x)
domain R = 0..n
var w: [R, 0..3]Int = 0
for r in R:
    for c in 0..3: w[r, c] = 10 * r.idx + c
print([x for ((r, c), x) in w.items if r == R.last or c == 0])
print(sum(i + j + k + x for ((i, j, k), x) in t.items), w.items.len)
