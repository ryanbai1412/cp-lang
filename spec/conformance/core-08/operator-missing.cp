// expect: E0606
// rule: 08.1.5
struct V2: x: Int, y: Int
fn (a: V2) + (b: V2) -> V2 = V2(a.x + b.x, a.y + b.y)
let q = V2(1, 2) * V2(3, 4) // <- here
