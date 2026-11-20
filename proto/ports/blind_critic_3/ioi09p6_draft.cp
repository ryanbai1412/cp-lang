// original: 5106126 (ioi09p6) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
let (n, kv): (Int, Int) = read()
domain R = 0..n
domain C = 0..n
let g: [R, C]Char = read()
let start = first((r, c) for r in R for c in C if g[r, c] == 'M')!
let home = first((r, c) for r in R for c in C if g[r, c] == 'D')!
var bee: [R, C]?Int = none
var q: Deque<(R, C)> = []
for r in R:
    for c in C:
        if g[r, c] == 'H':
            bee[r, c] = 0
            q.push_back((r, c))
while not q.is_empty:
    let p = q.pop_front()!
    for (r, c) in p.around():
        if (g[r, c] == 'G' or g[r, c] == 'M') and bee[r, c] == none:
            bee[r, c] = bee[p]! + 1
            q.push_back((r, c))
fn safe(p: (R, C), t: Int) -> Bool: return if let b = bee[p]: b > t else: true
fn check(t: Int) -> Bool:
    if not safe(start, t): return false
    var seen: [R, C]Bool = false
    var bfs: Deque<((R, C), Int)> = []
    seen[start] = true
    bfs.push_back((start, kv * t))
    while not bfs.is_empty:
        let (p, d) = bfs.pop_front()!
        if p == home: return true
        for (r, c) in p.around():
            if g[r, c] != 'T' and g[r, c] != 'H' and not seen[r, c] and safe((r, c), (d + 1) / kv):
                seen[r, c] = true
                bfs.push_back(((r, c), d + 1))
    return false
print(bsearch_last(0..=n * n, t => check(t)) ?? -1)
