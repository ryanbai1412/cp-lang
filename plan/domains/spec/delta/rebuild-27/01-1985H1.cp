// CF 1985H1: fill one row or column with '#', maximize the largest component.
// A row r joins every component touching rows r-1..r+1: a range add over rows.
@!multitest
let (n, m): (1..=1e6, 1..=1e6) = read()
domain R = 0..n
domain C = 0..m
let g = read<[R, C]Char>()

// Flood fill: comp[(r, c)] is the component id of a '#' cell.
var comp: [R, C]?Int = None
var size = List<Int>()
var lo_r = List<Int>()
var hi_r = List<Int>()
var lo_c = List<Int>()
var hi_c = List<Int>()
for r in R, c in C if g[r, c] == '#' and comp[r, c] == None:
    let k = size.len
    size.push(0)
    lo_r.push(Int(r))
    hi_r.push(Int(r))
    lo_c.push(Int(c))
    hi_c.push(Int(c))
    comp[r, c] = Some(k)
    var stack = [(r, c)]
    while let Some(p) = stack.pop():
        size[k] += 1
        lo_r[k] min= Int(p.0)
        hi_r[k] max= Int(p.0)
        lo_c[k] min= Int(p.1)
        hi_c[k] max= Int(p.1)
        for q in p.adj4 if g[q] == '#' and comp[q] == None:
            comp[q] = Some(k)
            stack.push(q)
var dots_r: [R]Int = 0
var dots_c: [C]Int = 0
for r in R, c in C if g[r, c] == '.':
    dots_r[r] += 1
    dots_c[c] += 1

var add_r: [0..=n]Int = 0
var add_c: [0..=m]Int = 0
for k in 0..size.len:
    add_r[max(lo_r[k] - 1, 0)] += size[k]
    add_r[min(hi_r[k] + 2, n)] -= size[k]
    add_c[max(lo_c[k] - 1, 0)] += size[k]
    add_c[min(hi_c[k] + 2, m)] -= size[k]

var best = 0
var run = 0
for r in R:
    run += add_r[Int(r)]
    best max= run + dots_r[r]
run = 0
for c in C:
    run += add_c[Int(c)]
    best max= run + dots_c[c]
print(best)
