// expect: E0405 at 3
struct E: u: Int, w: Int
let e = E(1, 2)
e.w = 3
