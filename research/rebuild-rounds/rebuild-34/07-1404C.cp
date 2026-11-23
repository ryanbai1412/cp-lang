// CF 1404C Fixed Point Removal: with left boundary l, element i is removable
// iff b = i - a[i] >= 0 and f(l) >= b, where f(l) counts removable elements in
// l..i-1 (non-increasing in l). Keep f as suffix sums of a Fenwick tree: add 1
// at the largest l that still works. Queries are answered offline by right end.
let n: 1..=3e5 = read()
let q: 1..=3e5 = read()
domain I = 1..=n
let a = read<[I]1..=n>()
domain Q = [q]
let qs = read<[Q](x: 0..n, y: 0..n)>()

var ending: [I]List<Q> = []
for j in Q: ending[I(n - qs[j].y)].push(j)

var f = Fenwick<I, Int>()
var ans: [Q]Int = 0
for i in I:
    let b = Int(i) - a[i]
    if b >= 0:
        if let l = (I.first!..=i).last_true(l => f.suffix(l.l) >= b):
            f.add(l, 1)
    for j in ending[i]:
        ans[j] = f.suffix(I(qs[j].x + 1).l)
print(ans, sep: "\n")
