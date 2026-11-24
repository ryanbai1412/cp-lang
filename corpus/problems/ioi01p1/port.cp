// Source: IOI 2001 P1 (Mobile Phones), https://dmoj.ca/problem/ioi01p1
// Problem: an S x S grid (S <= 1024) of counts, all 0. Commands: "0 S" starts, "1 X Y A" adds A
//   (possibly negative) to cell (X, Y), "2 L B R T" prints the sum over L <= X <= R,
//   B <= Y <= T, "3" ends. At most 60002 commands; cells stay in 0..=32767.
// Solution: a 2D Fenwick tree over X x Y: point add and rectangle sum in O(log^2 S) each.
// original: corpus/submissions/3477640.cpp

let (_, n): (0..=0, 1..=1024) = read()
domain X = 0..n
domain Y = 0..n

var grid = Fenwick2D<X, Y, Int>.new()

while true:
    let op: Int = read()
    if op == 1:
        let (x, y, a): (X, Y, -32767..=32767) = read()
        grid.add(x, y, a)
    elif op == 2:
        let (l, b, r, t): (X, Y, X, Y) = read()
        print(grid.fold(l..=r, b..=t))
    else:
        break
