// cp: 0.1 bea787c+dirty
type Edge = (u: Int, v: Int, w: Int)
let e: Edge = (1, 2, 3)
let (u, v, w) = e
print(e.u, u + v + w, e == e, (1, 2) < (1, 3))
