let (n, m) = read<(1..=1000, 1..=1000)>()
domain R = 0..n
domain C = 0..m
let g = read<[R, C]Char>()
var start = (R(0), C(0))
var goal = (R(0), C(0))
for r in R:
    for c in C:
        if g[r, c] == 'A':
            start = (r, c)
        if g[r, c] == 'B':
            goal = (r, c)

let res = bfs(p => p.adj4.filter(q => g[q] != '#'), from: [start], until: goal)
if let path = res.path(goal):
    print(yes(true))
    print(path.len - 1)
    var moves: List<Char> = []
    for i in 1..path.len:
        let (r1, c1) = path[i - 1]
        let (r2, c2) = path[i]
        if Int(r2) < Int(r1):
            moves.push('U')
        elif Int(r2) > Int(r1):
            moves.push('D')
        elif Int(c2) < Int(c1):
            moves.push('L')
        else:
            moves.push('R')
    print(moves, sep: "")
else:
    print(yes(false))
