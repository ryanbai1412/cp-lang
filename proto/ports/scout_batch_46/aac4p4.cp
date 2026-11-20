// AAC 4 P4 — original: corpus/submissions/4140376.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, m): (1..=50, 1..=50) = read()
domain Row = 1..=n
domain Col = 1..=m
let (s, f): (Row, Row) = read()
let (l, r): (0..=n * m, 0..=n * m) = read()
let arr: [Col]Row = read()

var on: [Row, Col]Bool = false
var cnt = 0

// switch cells on in order until l are on; false when the cells run out first
fn fill(cells: Iter<(Row, Col)>) -> Bool:
    for p in cells:
        if cnt >= l: break
        on[p] = true
        cnt += 1
    cnt >= l

fn solve() -> Bool:
    // FRICTION 46.1: a.find(x) only finds the first hit; the last one needs a reversed generator
    let hit = first(c for c in Col.rev if arr[c] == f)
    if hit == none:
        return s == f and fill((i, j) for i in Row if i != s for j in Col)
    let c = hit!
    if r == 0: return s == f
    if l <= n:
        on[s, c] = true
        cnt = 1
        return fill((i, c) for i in Row if i != s)
    for i in Row: on[i, c] = true
    cnt = n
    fill((i, j) for i in Row for j in Col if not on[i, j] and not (i == f and j > c))

if solve():
    print(cnt)
    // FRICTION 46.2: no spec'd way to print a Bool row as 0/1
    for i in Row: print(" ".join("{Int(on[i, j])}" for j in Col))
else:
    print(-1)
