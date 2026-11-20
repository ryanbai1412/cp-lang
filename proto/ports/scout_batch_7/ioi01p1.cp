// IOI 2001 Mobile Phones: an n by n grid of base stations, all at 0.
// "1 x y a" adds a to the count of cell (x, y); "2 l b r t" asks for the
// total in the rectangle l..=r by b..=t. "0 n" opens the session and "3"
// ends it.
//
// A Fenwick tree whose slots are Fenwick trees: the outer one walks the
// x axis, and each slot holds the y axis of the columns it covers.
use std.algebra.{Sum}
use std.fenwick.{Fenwick}

let (_, n): (Int, Int) = read()
domain X = 0..n
domain Y = 0..n

var slots = [Fenwick<Y, Int>(Sum<Int>) for _ in 0..=n]

fn add(x: X, y: Y, a: Int):
    var p = x.to_idx + 1
    while p <= n:
        slots[p].add(y, a)
        p += p & -p

fn left_of(x: Bounds<X>, ys: Range<Y>) -> Int:
    var (p, total) = (x.ord, 0)
    while p > 0:
        total += slots[p].fold(ys)
        p -= p & -p
    total

while true:
    let op: Int = read()
    if op == 1:
        let (x, y, a): (X, Y, Int) = read()
        add(x, y, a)
    elif op == 2:
        let (l, b, r, t): (X, Y, X, Y) = read()
        print(left_of(r.r, b..=t) - left_of(l.l, b..=t))
    else:
        break
