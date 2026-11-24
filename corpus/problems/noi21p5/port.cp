// Source: NOI 2021 P5 (Code), https://dmoj.ca/problem/noi21p5
// limits: 3000 ms, 1024 MB
// Problem: a string of n <= 1e5 ops W/E builds a sequence from (0, 1): W adds 1 to the last term;
//   E adds 1 to the second-to-last if the last is 1, else decrements it, appends 1, 1. Print the
//   continued fraction a0 + 1/(a1 + ...) as a b mod 998244353, then after each of q <= 1e5
//   updates: APPEND c, FLIP l r (swap W and E), REVERSE l r.
// Solution: W and E act on the fraction as the matrices [[1,0],[1,1]] and [[1,1],[0,1]], so the
//   answer is the row sums of their product. A std `SeqFold` keeps the sequence of matrices
//   with its product in both directions, so REVERSE needs no transpose trick; FLIP is a lazy
//   `Bool` tag that conjugates by the swap matrix (W <-> E), exchanging the diagonal entries and
//   the anti-diagonal ones. O(log n) expected per update.
// Bounds: n, q: 1..=1e5; l, r: 1..=n + q
// original: corpus/submissions/3927923.cpp

use std.lazy.{SeqFold}

type Z = Mod<998_244_353>
type M2 = (Z, Z, Z, Z)  // a 2 x 2 matrix, row by row

fn mul(a: M2, b: M2) -> M2 = (
    a.0 * b.0 + a.1 * b.2, a.0 * b.1 + a.1 * b.3,
    a.2 * b.0 + a.3 * b.2, a.2 * b.1 + a.3 * b.3,
)
fn step(e: Bool) -> M2 = if e: (Z(1), Z(1), Z(0), Z(1)) else: (Z(1), Z(0), Z(1), Z(1))

let (n, q): (1..=1e5, 1..=1e5) = read()
let s: Str = read()
let prod = Monoid<M2>((Z(1), Z(0), Z(0), Z(1)), mul)
let flip = Action<M2, Bool>(
    id: false,
    compose: (f, g) => f != g,
    apply: (f, m, _) => if f: (m.3, m.2, m.1, m.0) else: m,
    over: prod,
)
var ops = SeqFold<M2, Bool>(prod, [step(c == 'E') for c in s], act: Some(flip))

fn report():
    let m = ops.total
    print(m.0 + m.1, m.2 + m.3)

report()
repeat q:
    match read<Str>():
        "APPEND" => ops.push(step(read<Str>() == "E"))
        kind =>
            let (l, r): (1..=n + q, 1..=n + q) = read()
            if kind == "FLIP": ops.apply(l - 1..r, true) else: ops.reverse(l - 1..r)
    report()
