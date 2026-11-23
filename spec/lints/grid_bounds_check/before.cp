let (h, w): (1..=1000, 1..=1000) = read()
let g: [h, w]Char = read()
let (x, y): (0..h, 0..w) = read()
for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
    let (nx, ny) = (x + dx, y + dy)
    if nx >= 0 and nx < h and ny >= 0 and ny < w:
        if g[nx, ny] == '.':
            print(nx, ny)
