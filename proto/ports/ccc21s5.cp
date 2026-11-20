// CCC 2021 S5 Math Homework: find a[1..n] such that gcd(a[l..r]) = x for
// every requirement, or report that none exists. Each a[i] must be a multiple
// of every x that covers it, so take the lcm; then just check the gcds.
let (n, q): (1..=150000, 1..=100000) = read()
domain I = 1..=n
domain X = 1..=16
let reqs: [0..q](l: I, r: I, x: X) = read()

// opens[x][i] - closes[x][i]: requirements with value x that start / end at i
var opens: [X, I]Int = 0
var closes: [X, I]Int = 0
for (l, r, x) in reqs:
    opens[x, l] += 1
    closes[x, r] += 1

var a: [I]Int = 1
for x in X:
    var open = 0
    for i in I:
        open += opens[x, i]
        if open > 0: a[i] = lcm(a[i], Int(x))
        open -= closes[x, i]

let gcds = SegTree<I, Int>(gcd, a)
if all(gcds.fold(l..=r) == Int(x) for (l, r, x) in reqs): print(a) else: print("Impossible")
