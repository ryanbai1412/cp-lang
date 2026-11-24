let a: Pt = Pt(1, 2)
let b = Pt(3, 4)
print((a + b).x, dot(a, b), cross(a, b), a.norm2, turn(a, b, Pt(0, 0)))
