struct Cell(r: Int, c: Int)

impl Read for Cell:
    fn read() -> Cell:
        let r: Int = read()
        let c: Int = read()
        return Cell(r - 1, c - 1)

@allow(unread_field)
struct Move(at: Cell, d: Char)

let n: Int = read()
let start: Cell = read()
let moves: [n]Move = read()
print(start)
for m in moves:
    print(m.at.r, m.at.c, m.d)
