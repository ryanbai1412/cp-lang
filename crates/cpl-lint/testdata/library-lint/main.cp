use mylib.geo
let xa: Int = read()
let xb: Int = read()
let d2 = (xa - xb) * (xa - xb)
let e = xa * xb
let f = xa * xa + 1
let g: Int = read() * read()
with lint(square_by_hand: allow):
    print(d2, e, f, g, xb * xb)
