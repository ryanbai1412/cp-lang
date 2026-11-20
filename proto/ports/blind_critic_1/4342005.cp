// Corpus submission 4342005: multi-source bounded-distance grid BFS (sorted by color)
let n: 1..=1500 = read()
let m: 1..=1500 = read()
let k: 0..=1e6 = read()

domain Row = 1..=n
domain Col = 1..=m

var grid: [Row, Col]Int = read()
var vis: [Row, Col]Bool = false
var q: List<(r: Row, c: Col, color: Int, d: Int)> = []

fn push_cell(r: Row, c: Col, color: Int, d: Int):
    if d <= k and not vis[r, c]:
        vis[r, c] = true
        q.push((r, c, color, d))

var ord: List<(color: Int, r: Row, c: Col)> = []
for r in Row:
    for c in Col:
        if grid[r, c] != 0:
            ord.push((grid[r, c], r, c))
ord.sort()

for item in ord:
    push_cell(item.r, item.c, item.color, 0)

var head = 0
while head < q.len:
    // Copy by value via `var` to avoid dangling const-ref across `q.push` reallocations
    var cur = q[head]
    head += 1
    if grid[cur.r, cur.c] == 0:
        grid[cur.r, cur.c] = cur.color
    if let nr = cur.r.offset(-1):
        push_cell(nr, cur.c, cur.color, cur.d + 1)
    if let nr = cur.r.offset(1):
        push_cell(nr, cur.c, cur.color, cur.d + 1)
    if let nc = cur.c.offset(-1):
        push_cell(cur.r, nc, cur.color, cur.d + 1)
    if let nc = cur.c.offset(1):
        push_cell(cur.r, nc, cur.color, cur.d + 1)

for r in Row:
    var row_out: [Col]Int = [Col] c => grid[r, c]
    print(row_out)
