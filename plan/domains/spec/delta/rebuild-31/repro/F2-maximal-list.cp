let a = [1, 1, 1]
let g = a.maximal(Window(+), ok: w => (w.all ?? 0) <= 1)
print(g[2].len)
