// UCRPC 2021 D: an ASCII maze of n x m rooms drawn on a (2n+1) x (2m+1)
// canvas with '+', '-', '|' walls. Print the sizes of the four largest
// connected regions of rooms, largest first.
// original: corpus/submissions/3944584.cpp
let (n, m): (1..=500, 1..=500) = read()
domain Row = 0..=2 * n
domain Col = 0..=2 * m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

fn wall(p: Cell) -> Bool: g[p] in "+-|"

// Rooms sit on odd canvas cells; a door is an open cell between two rooms.
var rooms = Dsu<Cell>()
for i in (1..2 * n).step(2):
    for j in (1..2 * m).step(2):
        let p: Cell = (Row(i), Col(j))        // FRICTION D1: odd-cell sub-lattice of a canvas
        for d in Dir4:
            if let door = p.step(d) and let q = door.step(d):
                if not wall(door): rooms.unite(p, q)

var sizes = rooms.components.map(c => c.len).sorted(by: s => -s)
print(sizes[0], sizes[1], sizes[2], sizes[3])
