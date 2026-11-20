// NOI '19 P5 - Landlords (DMOJ: noi19p5, 25pt, TL 1.0s, ML 511MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Deck of n cards starts with f(i) = i^kind (kind in {1, 2}) for i = 1..n.
// Each shuffle splits into top A cards and bottom B = n - A cards and riffle-shuffles
// uniformly over all C(n, A) interleavings. Answer Q queries for E[card at c_i] mod 998244353.
//
// Key invariant: uniform riffle shuffles preserve polynomial degree <= 2!
// Track f(x) = a*x^2 + b*x + c across every shuffle in O(1) by computing the exact
// expected values g(1), g(2), g(n) after the shuffle and interpolating the new (a, b, c).

type Z = Mod<998244353>

let (n, m, kind): (1..=1e7, 0..=5e5, 1..=2) = read()
domain Shuffle = 0..m
let cuts: [Shuffle](0..=n) = read()
let q: 1..=5e5 = read()
domain Query = 0..q
let asks: [Query](1..=n) = read()

// Polynomial f(x) = a * x^2 + b * x + c over Mod<998244353>
// WISH MG6: Poly<Z> with eval(x) and interpolate(pts) in std.poly
var (a, b, c) = if kind == 1: (Z(0), Z(1), Z(0)) else: (Z(1), Z(0), Z(0))

fn eval(a: Z, b: Z, c: Z, x: Int) -> Z:
    let zx = Z(x)
    (a * zx + b) * zx + c

if n == 1:
    for i in Query: print(1)
    return

let inv_n = Z(n).inv
let inv_n1 = Z(n - 1).inv
let inv_n_n1 = inv_n * inv_n1

if n == 2:
    var (y1, y2) = (eval(a, b, c, 1), eval(a, b, c, 2))
    for s in Shuffle:
        let k = cuts[s]
        if k == 1:
            let avg = (y1 + y2) * inv_n
            (y1, y2) = (avg, avg)
    for i in Query: print(if asks[i] == 1: y1 else: y2)
    return

let inv_n2 = Z(n - 2).inv

for s in Shuffle:
    let k = cuts[s]
    let r = n - k
    // WISH MG4: constant/Int -> Mod<P> promotion would eliminate Z(k), Z(r), Z(k - 1), Z(r - 1)
    let zk = Z(k)
    let zr = Z(r)
    let f1 = eval(a, b, c, 1)
    let f2 = eval(a, b, c, 2)
    let fk = eval(a, b, c, k)
    let fk1 = eval(a, b, c, k + 1)
    let fk2 = eval(a, b, c, k + 2)
    let fn_ = eval(a, b, c, n)

    let g1 = (zk * f1 + zr * fk1) * inv_n
    let gn = (zk * fk + zr * fn_) * inv_n
    let g2 = (zk * Z(k - 1) * f2 + zk * zr * (f1 + fk1) + zr * Z(r - 1) * fk2) * inv_n_n1

    // Recover quadratic g(x) = a*x^2 + b*x + c from (1, g1), (2, g2), (n, gn):
    // Divided differences: d12 = g2 - g1, d1n = (gn - g1) / (n - 1), a = (d1n - d12) / (n - 2)
    let d12 = g2 - g1
    let d1n = (gn - g1) * inv_n1
    a = (d1n - d12) * inv_n2
    b = d12 - 3 * a
    c = g1 - a - b

for i in Query:
    print(eval(a, b, c, asks[i]))
