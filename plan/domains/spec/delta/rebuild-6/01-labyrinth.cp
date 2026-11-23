// CSES 1193 Labyrinth: shortest path from A to B in a grid, with the moves.
// Input: n m, then n rows of '.', '#', 'A', 'B'. Output: NO, or YES, the length, the moves in L/R/U/D.
domain R = 0..read<1..=1000>()
domain C = 0..read<1..=1000>()
let grid = read<[R, C]Char>()
let a = grid.keys.find(p => grid[p] == 'A')!
let b = grid.keys.find(p => grid[p] == 'B')!

fn move(x: (R, C), y: (R, C)) -> Char:
    match (Int(y.0) - Int(x.0), Int(y.1) - Int(x.1)):
        (-1, 0) => 'U'
        (1, 0) => 'D'
        (0, -1) => 'L'
        _ => 'R'

let paths = bfs(p => p.adj4.filter(q => grid[q] != '#'), from: [a], until: b)
let path = paths.path(b) else:
    print("NO")
    return
print("YES")
print(paths[b]!)
print("".join(move(x, y) for (x, y) in path.adjacent))
