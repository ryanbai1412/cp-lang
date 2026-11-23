// expect: ok, output "5"
// rule: 04.2.20
// input: "..A\n...\n"
// An inline product domain `(R, C)` is a `Seq` of cells, row-major (rebuild round 6 H-GRID-FIND).
domain R = 0..2
domain C = 0..3
let g = read<[R, C]Char>()
print((R, C).count(p => g[p] == ('.')))
