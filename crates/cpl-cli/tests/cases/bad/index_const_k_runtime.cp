// error: `const` is compile-time only
let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
struct Cell:
    r: Row
    c: Col
impl Index for Cell:
    const count = h * w
    @get fn ord(self) -> Int = r.ord * w + c.ord
    fn nth(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))
