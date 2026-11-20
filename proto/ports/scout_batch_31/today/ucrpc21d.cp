// UCRPC 2021 D: an ASCII maze of n x m rooms drawn on a (2n+1) x (2m+1)
// canvas with '+', '-', '|' walls. Print the sizes of the four largest
// connected regions of rooms, largest first.
// Runnable on proto/cpc today.
// original: corpus/submissions/3944584.cpp
let n: 1..=500 = read()
let m: 1..=500 = read()
domain Row = 0..=2 * n
domain Col = 0..=2 * m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

fn wall(p: Cell) -> Bool: g[p] == '+' or g[p] == '-' or g[p] == '|'

// The room two canvas cells away in each direction, when the canvas cell
// between them is open.
fn doors(p: Cell) -> List<Cell>:
    var out: List<Cell> = []
    if let a = p.0 - 1 and let b = p.0 - 2:
        if not wall((a, p.1)): out.push((b, p.1))
    if let a = p.0 + 1 and let b = p.0 + 2:
        if not wall((a, p.1)): out.push((b, p.1))
    if let a = p.1 - 1 and let b = p.1 - 2:
        if not wall((p.0, a)): out.push((p.0, b))
    if let a = p.1 + 1 and let b = p.1 + 2:
        if not wall((p.0, a)): out.push((p.0, b))
    out

var seen: [Row, Col]Bool = false
var sizes: List<Int> = []
for i in 0..n:
    for j in 0..m:
        let start: Cell = (Row(2 * i + 1), Col(2 * j + 1))
        if seen[start]: continue
        seen[start] = true
        var stack: List<Cell> = [start]
        var size = 0
        while let p = stack.pop():
            size += 1
            for q in doors(p):
                if not seen[q]:
                    seen[q] = true
                    stack.push(q)
        sizes.push(size)
sort(mut sizes, by: x => -x)
print(sizes[0], sizes[1], sizes[2], sizes[3])
