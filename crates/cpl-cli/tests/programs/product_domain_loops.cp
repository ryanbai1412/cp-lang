enum Dir:
    Up
    Down
    Left
let h: 1..=4 = read()
let w: 1..=4 = read()
domain Row = 0..h
domain Col = 0..w
domain Cell = (Row, Col)
domain Turn = (r: Row, d: Dir, ok: Bool)
let g: [Row, Col]Char = read()

var hits = 0
for (r, c) in Cell if g[r, c] == '#':
    hits += 1
print(hits)

var first = "none"
for (r, c) in Cell:
    if g[r, c] == '#':
        first = "{r} {c}"
        break
print(first)

var seen = 0
'outer: for (r, c) in Cell:
    if Int(c) == 1: continue
    for k in 0..3:
        if k == 1 and Int(r) == 1: break 'outer
        seen += 1
print(seen)

var turns = 0
for (r, d, ok) in Turn if ok and d != Dir.Left:
    turns += Int(r) + 1
print(turns)

for (i, j) in Row.pairs:
    if Int(j) == 2: break
    print(i, j)

var anon = 0
for (r, d) in (Row, Dir) if d == Dir.Down:
    anon += Int(r) * 10 + 1
print(anon)
