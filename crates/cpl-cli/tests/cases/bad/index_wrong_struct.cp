// error: index of type
let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
struct Cell:
    r: Row
    c: Col
impl Index for Cell:
    let K = h * w
    fn to_idx(self) -> Int = r.ord * w + c.ord
    fn from_idx(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))
struct Other:
    x: Row
var d: [Cell]Int = 0
d[Other(Row.first)] = 1
