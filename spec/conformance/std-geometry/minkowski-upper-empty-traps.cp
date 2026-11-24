// expect: R0401
let a: List<Pt> = []
print(minkowski_upper(a, [Pt(0, 0)]).len) // <- here
