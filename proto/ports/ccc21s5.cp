// cp: 0.1 f7ce0ce+dirty
// CCC 2021 S5 Math Homework: find a[1..n] such that gcd(a[l..r]) = x for
// every requirement, or report that none exists. Each a[i] must be a multiple
// of every x that covers it, so take the lcm; then just check the gcds.
let n: 1..=150000 = read()
let q: 1..=100000 = read()
domain I = 1..=n
domain X = 1..=16
let reqs: [q](l: I, r: I, x: X) = read() where all(t.l <= t.r for t in reqs)

// cover[x, b]: requirements with value x open at bound b (difference, then prefix)
var cover: [X, I.bounds]Int = 0
for (l, r, x) in reqs:
    cover[x, l.before] += 1
    cover[x, r.after] -= 1
var gcds = SegTree<I, Int>(0, (a, b) => gcd(a, b))
var a: [I]Int = 1
for x in X:
    var open = 0
    for i in I:
        open += cover[x, i.before]
        if open > 0: a[i] = lcm(a[i], x.label)
for i in I: gcds.set(i, a[i])

if all(gcds.query(l, r) == x.label for (l, r, x) in reqs):
    print(a)
else:
    print("Impossible")
