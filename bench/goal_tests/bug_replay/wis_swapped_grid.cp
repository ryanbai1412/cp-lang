let (h, w): (Int, Int) = read()
domain R = 0..h
domain C = 0..w
let g: [R, C]Int = read()
for r in R:
    for c in C:
        print(g[c, r])
