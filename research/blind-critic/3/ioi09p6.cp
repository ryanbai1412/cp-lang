// original: 5106126 (ioi09p6)
// IOI 2009 Mecho: longest time the bear can keep eating honey and still reach home ahead of
// the bees. Multi-source BFS gives each cell's bee arrival minute; binary search the wait
// with a BFS that only steps onto cells the bees have not reached yet.
let n: Int = read()
let kv: Int = read()
domain R = 0..n
domain C = 0..n
let g: [R, C]Char = read()

fn around(r: R, c: C) -> List<(R, C)>:
    var out: List<(R, C)> = []
    if let u = r - 1: out.push((u, c))
    if let d = r + 1: out.push((d, c))
    if let w = c - 1: out.push((r, w))
    if let e = c + 1: out.push((r, e))
    out

var start = (R.first, C.first)
var home = (R.first, C.first)
var bee: [R, C]?Int = none
var hives: List<(R, C)> = []
for r in R:
    for c in C:
        if g[r, c] == 'M': start = (r, c)
        if g[r, c] == 'D': home = (r, c)
        if g[r, c] == 'H':
            bee[r, c] = 0
            hives.push((r, c))
var head = 0
while head < hives.len:
    let (pr, pc) = hives[head]
    head += 1
    for (r, c) in around(pr, pc):
        if (g[r, c] == 'G' or g[r, c] == 'M') and bee[r, c] == none:
            bee[r, c] = bee[pr, pc]! + 1
            hives.push((r, c))

fn safe(r: R, c: C, t: Int) -> Bool:
    if let b = bee[r, c]: return b > t
    true

fn check(t: Int) -> Bool:
    if not safe(start.0, start.1, t): return false
    var seen: [R, C]Bool = false
    var queue: List<(R, C, Int)> = []
    seen[start.0, start.1] = true
    queue.push((start.0, start.1, kv * t))
    var at = 0
    while at < queue.len:
        let (pr, pc, d) = queue[at]
        at += 1
        if (pr, pc) == home: return true
        for (r, c) in around(pr, pc):
            if g[r, c] != 'T' and g[r, c] != 'H' and not seen[r, c] and safe(r, c, (d + 1) / kv):
                seen[r, c] = true
                queue.push((r, c, d + 1))
    false

var lo = -1                                 // largest wait known to work
var hi = 1e9 + 1                           // the original caps the answer at 1e9 when bees never arrive
while hi - lo > 1:
    let mid = (lo + hi) / 2
    if check(mid): lo = mid else: hi = mid
print(lo)
