// expect: ok, output "1 4 2 3 x YES YES\n6 7"
// rule: R10.2.4
// input: "2 5\n7 8\n"
// A `Read` impl reads its parts from the `Input` it is given, so `read(from:)` reads them from the
// string, and standard input stays where it was.
struct Cell(r: Int, c: Int)
impl Read for Cell:
    fn read(mut input: Input) -> Cell:
        let r: 1..=9 = input.read()
        let c = input.read<Int>()
        Cell(r - 1, c - 1)
let a: Cell = read()
let b = read<(Cell, Str)>(from: "3 4 x")!
print(a.r, a.c, b.0.r, b.0.c, b.1, yes(read<Cell>(from: "1") == None), yes(read<[2]Cell>(from: "1 2 3 4 5") == None))
let z: Cell = read()
print(z.r, z.c)
