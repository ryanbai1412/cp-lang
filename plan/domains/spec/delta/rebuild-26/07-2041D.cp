// 2041D Drunken Maze: BFS over (cell, last direction, run length so far).
let (n, m): (3..=1e4, 3..=1e4) = read()
domain R = 0..n
domain C = 0..m
domain Cell = (R, C)
domain Way = 0..4
domain Run = 0..=3
domain State = (R, C, Way, Run)
let maze = read<[R, C]Char>()
let start = Cell.find(p => maze[p] == 'S')!
let goal = Cell.find(p => maze[p] == 'T')!

fn moves(s: State) -> yields State:
    let (r, c, w, run) = s
    for nw in Way:
        if nw == w and Int(run) == 3: continue
        let q = (r, c).step(Dir4(Int(nw)))
        if q == None or maze[q!] == '#': continue
        let next_run = if nw == w: Int(run) + 1 else: 1
        yield (q!.0, q!.1, nw, Run(next_run))

let paths = bfs(moves, from: [(start.0, start.1, Way(0), Run(0))])
var best: ?Int = None
for w in Way:
    for k in Run:
        best min= paths.get((goal.0, goal.1, w, k))
print(best ?? -1)
