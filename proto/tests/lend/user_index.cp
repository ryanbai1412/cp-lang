let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w

// any struct can be a bounded index: K is frozen from `h * w` when the impl is declared
struct Cell:
    r: Row
    c: Col

impl Index for Cell:
    let K = h * w
    fn to_idx(self) -> Int = r.ord * w + c.ord
    fn from_idx(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))

var dist: [Cell]Int = 0
for x in Cell:
    dist[x] = x.r.label * 10 + x.c.label
let last = Cell(Row.last, Col.last)
print(Cell.K, dist[last], last.to_idx, Cell.from_idx(1).c)
print(dist)
