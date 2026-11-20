// COCI '25 Contest 3 P1: a brush starts at the centre of an n x n canvas.
// L R U D move it one cell (it stays put at an edge); any other letter
// paints, in that colour, every cell at Manhattan distance below k.
// Runnable today on the interpreter (`cp.py lib-run`): proto/cpc cannot yet
// iterate or index a Str.
// original: corpus/submissions/7730233.cpp
let n: 1..=100 = read()
let k: 1..=200 = read()
let moves: Str = read()
domain Row = 0..n
domain Col = 0..n
var g: [Row, Col]Char = '.'
var r = Row(n / 2)
var c = Col(n / 2)
for ch in moves:
    if ch == 'L':
        if let x = c - 1: c = x
    elif ch == 'R':
        if let x = c + 1: c = x
    elif ch == 'U':
        if let x = r - 1: r = x
    elif ch == 'D':
        if let x = r + 1: r = x
    else:
        for i in Row:
            for j in Col:
                if abs(i.to_idx - r.to_idx) + abs(j.to_idx - c.to_idx) < k: g[i, j] = ch
for i in Row:
    var line = ""
    for j in Col: line.push(g[i, j])
    print(line)
