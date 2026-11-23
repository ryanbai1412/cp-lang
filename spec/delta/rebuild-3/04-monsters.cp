// CSES 1194 Monsters: walk from A to the grid edge, never on a cell a monster can reach first.
// In: n m, then n rows of m chars from "#.AM". Out: YES, the path length and its moves (UDLR), or NO.
domain R = 0..read<1..=1000>()
domain C = 0..read<1..=1000>()
let grid = read<[R, C]Char>()
let open = (p: (R, C)) => p.adj4.filter(q => grid[q] != '#')
let monster = bfs(open, from: (R, C).filter(p => grid[p] == 'M'))
let start = (R, C).find(p => grid[p] == 'A')!
let me = bfs(open, from: [start], enter: (p, d) => monster[p].is_none_or(m => d < m))

fn on_edge(p: (R, C)) -> Bool:
    let (r, c) = p
    r == R.first! or r == R.last! or c == C.first! or c == C.last!

let exit = (R, C).find(p => on_edge(p) and me.reached(p)) else:
    print("NO")
    return
let path = me.path(exit)!
var moves = ""
for (p, q) in path.adjacent:
    for (d, letter) in zip(Dir4.all(), "URDL") if p.step(d) == Some(q): moves.push(letter)
print("YES")
print(path.len - 1)
print(moves)
