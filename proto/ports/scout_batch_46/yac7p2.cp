// YAC 7 P2 — original: corpus/submissions/5705152.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

@multitest
let (n, m): (1..=1000, 1..=1000) = read()
if n == 1 or m == 1:
    print(-1)
    return

// an even side must be the height; transpose when only m is even
let flip = n % 2 == 1 and m % 2 == 0
let (h, w) = if flip: (m, n) else: (n, m)
domain Row = 0..h
domain Col = 0..w
var g: [Row, Col]Int = 0
var at = (Row.first, Col.first)
var k = 1
g[at] = 1

// walk cnt cells in direction d, numbering each one
// FRICTION 46.7: the spec names Dir4 but never its four variants
fn go(d: Dir4, cnt: Int):
    for _ in 0..cnt:
        at = at.step(d)!
        k += 1
        g[at] = k

if h % 2 == 0:
    // snake through columns 1.., come home along column 0
    go(R, w - 1)
    for _ in 0..(h - 2) / 2:
        go(D, 1)
        go(L, w - 2)
        go(D, 1)
        go(R, w - 2)
    go(D, 1)
    go(L, w - 1)
    go(U, h - 2)
else:
    // both sides odd: the original's open path
    go(R, w - 1)
    go(D, h - 1)
    for _ in 0..(w - 3) / 2:
        go(L, 1)
        go(U, h - 2)
        go(L, 1)
        go(D, h - 2)
    go(L, 1)
    for _ in 0..(h - 3) / 2:
        go(L, 1)
        go(U, 1)
        go(R, 1)
        go(U, 1)
    go(L, 1)
    go(U, 1)
    go(R, 1)

// FRICTION 46.8: g and g.transposed have different types, so one binding cannot hold either
if flip:
    let out = g.transposed
    for c in Col: print(out.row(c))
else:
    for r in Row: print(g.row(r))
