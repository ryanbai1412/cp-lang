let n: Int = read()
domain V = 0..n

// Whole rows: every cell takes the larger (or smaller) of the two.
var a: [V]Int = [V] v => v.idx
var b: [V]Int = [V] v => 3 - v.idx
print(a max= b, a)
print(a max= b, a)
print(a min= b, a)
a min= [V] v => 1
print(a)
var c: [0..n]Int = 5
var d: [0..n]Int = [4, 9, 5, 1]
print(c min= d, c)
c max= d
print(c)
var f: [1..=3]F64 = [1.5, 2.5, 3.5]
let lo: [1..=3]F64 = [2.0, 2.0, 2.0]
if f max= lo: print(f)
var xs = [1, 5, 3]
xs max= [4, 4, 4]
print(xs)
var rows: [0..2]List<Int> = [0, 0, 9]
rows[1] max= xs
print(rows)
var s = 3
s max= 9
print(s)

// A tuple taken apart by `if let` and `while let`.
var m = Map<Int, Int>()
m[3] = 4
m[1] = 2
let opt: ?(Int, Int) = (1, 2)
if let (x, y) = opt: print(x, y)
if let (x, y) = opt, x > y: print("no")
else: print("guard")
while let (k, v) = m.pop_first(): print(k, v)
var st: List<(Int, Str)> = [(1, "a"), (2, "b"), (3, "c")]
while let (i, s) = st.pop(), i > 1: print(i, s)
print(st)
let pair: ?((Int, Int), Str) = ((1, 2), "z")
if let ((p, q), name) = pair: print(p, q, name)

// `for` over the cells of a grid.
domain R = 0..2
domain K = 0..3
var g: [0..2, 0..3]Int = [[1, 2, 3], [4, 5, 6]]
var t = 0
for ((r, c), x) in g.items: t += r * 100 + c * 10 + x
print(t)
var h: [R, K]Int = 0
for r in R: for k in K: h[r, k] = r.idx * 3 + k.idx
for ((r, k), x) in h.items:
    if k == K.first: continue
    if r == R.last and k == K.last: break
    print(r, k, x)
var q: [1..=2, 1..=2, 0..2]Int = 7
for (at, x) in q.items: print(at, x)
var e: [0..0, 0..3]Int = 0
for (at, x) in e.items: print("never", at, x)
print([x for ((r, c), x) in g.items if r == c])

// `last(k)` on an array indexed by the bounds of a domain.
var w: [V.bounds]Int = 0
for v in V: w[v.r] = w[v.l] + v.idx + 1
print(w)
print(w.last, w.last(0), w.last(1), w.last(n), w.last(n + 1), w.last(-1))
print(w.last(1)! + 1, w.last ?? -1)
