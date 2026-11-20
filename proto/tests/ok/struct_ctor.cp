// cp: 0.1 0b3d014+dirty
struct P:
    x: Int
    y: 0..=100
let p = P(x: 1, y: 2)
let q = P(5, 7)
let r = P(y: 3, x: 4)
print(p.x + p.y, q.y, r.x)
