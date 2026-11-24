// A read whose spec reads through a type's own `impl Read` runs on the VM
// as a read sequence: the call alone, inside a tuple and a struct, as the
// cells of an array, behind an optional, and over a string.
struct Cell(r: Int, c: Int)

impl Read for Cell:
    fn read() -> Cell:
        let r: Int = read()
        let c: Int = read()
        return Cell(r - 1, c - 1)

with lint(unread_field: allow)
struct Move(at: Cell, d: Char)

let start: Cell = read()
let pair: (Int, Cell) = read()
let n: Int = read()
let moves: [n]Move = read()
let grid: [2][2]Cell = read()
let far = read<Cell>(from: "5 7")
let short = read<Cell>(from: "5")
let left = read<Cell>(from: "5 7 9")
let last = read<?Cell>()
let gone = read<?Cell>()
print(start, pair.0, pair.1)
for m in moves:
    print(m.at.r, m.at.c, m.d)
print(grid[0][1], grid[1][0])
print(far!, yes(short == None), yes(left == None), last!, yes(gone == None))
