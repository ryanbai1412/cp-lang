// error: can't be sized by the var `m`
let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
struct Cell:
    r: Row
    c: Col
var m = h * w
impl Index for Cell:
    let K = m
    fn to_idx(self) -> Int = r.ord * w + c.ord
    fn from_idx(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))
