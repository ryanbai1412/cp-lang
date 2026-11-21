// CCC 2021 S5 Math Homework: find a[1..n] such that gcd(a[l..r]) = x for
// every requirement, or report that none exists. Each a[i] must be a multiple
// of every x that covers it, so take the lcm; then just check the gcds.
let (n, q): (1..=150000, 1..=100000) = read()
domain I = 1..=n
let reqs: [0..q](l: I, r: I, x: 1..=16) = read()

// delta[x, b]: change at boundary b in how many requirements with value x are open
var delta: [0..=16, I.cuts]Int = 0
for (l, r, x) in reqs:
    delta[x, l.l] += 1
    delta[x, r.r] -= 1

var a: [I]Int = 1
for x in 1..=16:
    var open = 0
    for i in I:
        open += delta[x, i.l]
        if open > 0: a[i] = lcm(a[i], x)

let gcds = SegTree<I, Int>(gcd, a)
if all(gcds.fold(l..=r) == x for (l, r, x) in reqs): print(a) else: print("Impossible")
