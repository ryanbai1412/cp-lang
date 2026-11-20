domain R = 0..3
var f = Fenwick2D<R, R, Int>()
f.add(R(1), R(1), 5)
print(f.sum(R(0)..=R(2), R(0)..=R(2)))
