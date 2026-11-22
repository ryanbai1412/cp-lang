let (h, w): (1..=50, 1..=50) = read()
domain Row = 1..=h
domain Col = 1..=w
let g: [Row, Col]Char = read()
var best = 0
for r in Row:
    var run = 0
    for c in Col:
        if g[r, c] == '.':
            run += 1
            best max= run
        else:
            run = 0
print(best)
