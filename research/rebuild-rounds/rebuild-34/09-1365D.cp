// CF 1365D Solve The Maze: wall in every bad person's neighbours (fails if a
// good person is adjacent), then every good person must reach the exit.
@!multitest
let n: 1..=50 = read()
let m: 1..=50 = read()
domain R = 1..=n
domain C = 1..=m
var grid = read<[R, C]Char>()

var ok = true
let cells = [(r, c) for r in R for c in C]
for p in cells:
    if grid[p] != 'B': continue
    for q in p.adj4:
        if grid[q] == 'G': ok = false
        if grid[q] == '.': grid[q] = '#'

let open = (c: (R, C)) => grid[c] != '#'
let reach = bfs(c => c.adj4.filter(open), from: [(R.last!, C.last!)], enter: (c, _) => open(c))
ok = ok and cells.all(p => grid[p] != 'G' or reach[p] != None)
print(yes(ok, answers: ("Yes", "No")))
