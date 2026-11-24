// expect: L0202
struct Pt(x: Int, y: Int) // <- here
let p = Pt(x: 1, y: 2)
print(p.x)
