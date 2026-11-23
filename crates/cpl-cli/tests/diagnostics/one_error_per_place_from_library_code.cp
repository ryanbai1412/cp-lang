domain X = 0..4
var g = Fenwick2D<X, X, Int>()
let (l, b, r, t): (X, X, X, X) = (0, 0, 1, 1)
print(g.fold(l..=r, b..=t))
