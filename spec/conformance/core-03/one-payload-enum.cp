// expect: ok, output "-1 -2 4"
// rule: 03.2.4
domain V = 0..5
enum Cell:
    Wall
    Empty
    Box(id: V)
fn code(c: Cell) -> Int = match c: Wall => -1, Empty => -2, Box(id) => Int(id)
print(code(Cell.Wall), code(Cell.Empty), code(Cell.Box(V(4))))
