domain P = 0..3
let a: [P]Int = [P] _ => 1
let g = a.maximal(Window(+), ok: w => (w.all ?? 0) <= 1)
print(g[P.last!].len)
