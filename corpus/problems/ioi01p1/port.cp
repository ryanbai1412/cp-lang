// IOI 2001 Mobile Phones: an n by n grid of base stations, all at 0.
// "1 x y a" adds a to the count of cell (x, y); "2 l b r t" asks for the
// total in the rectangle l..=r by b..=t. "0 n" opens the session and "3"
// ends it.

let (_, n): (Int, Int) = read()
domain X = 0..n
domain Y = 0..n

var grid = Fenwick2D<X, Y, Int>.new()

while true:
    let op: Int = read()
    if op == 1:
        let (x, y, a): (X, Y, Int) = read()
        grid.add(x, y, a)
    elif op == 2:
        let (l, b, r, t): (X, Y, X, Y) = read()
        print(grid.fold(l..=r, b..=t))
    else:
        break
