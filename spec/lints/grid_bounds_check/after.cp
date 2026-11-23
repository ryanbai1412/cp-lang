let (h, w): (1..=1000, 1..=1000) = read()
domain Row = 0..h
domain Col = 0..w
let g: [Row, Col]Char = read()
let (x, y): (Row, Col) = read()
for (nx, ny) in (x, y).adj4:
    if g[nx, ny] == '.':
        print(nx, ny)
