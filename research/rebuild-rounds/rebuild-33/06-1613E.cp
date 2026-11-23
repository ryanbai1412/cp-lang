// cf1613e Crazy Robot: BFS back from the lab; a free cell becomes winning once at most one of
// its free neighbours is not yet winning.
@!multitest
let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain R = 0..n
domain C = 0..m
domain Cell = (R, C)
var g = read<[R, C]Char>()
let lab = Cell.find(p => g[p] == 'L')!

var open: [R, C]Int = [R, C] (r, c) => (r, c).adj4.count(q => g[q] != '#')
var queue = [lab]
var head = 0
while head < queue.len:
    let u = queue[head]
    head += 1
    for v in u.adj4 if g[v] == '.':
        open[v] -= 1
        if open[v] <= 1:
            g[v] = '+'
            queue.push(v)
for row in g.rows: print(row)
