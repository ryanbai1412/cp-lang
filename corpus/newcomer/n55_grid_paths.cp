type Z = Mod<1_000_000_007>
let (h, w): (2..=1000, 2..=1000) = read()
let grid: [h, w]Char = read()
var ways: [h, w]Z = 0
ways[0, 0] = 1
for r in 0..h:
    for c in 0..w:
        if grid[r, c] == '#':
            continue
        if r > 0:  // expect: grid_neighbour_step
            ways[r, c] += ways[r - 1, c]
        if c > 0:  // expect: grid_neighbour_step
            ways[r, c] += ways[r, c - 1]
print(ways[h - 1, w - 1])
