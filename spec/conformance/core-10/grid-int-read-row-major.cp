// expect: ok, output "4 3"
// rule: 10.2.3
// input: "1 2\n3\n4 5 6\n"
// `[R, C]T` reads R*C tokens row-major wherever the line breaks fall (rebuild round 16 H-GRID-INT-READ).
domain R = 0..2
domain C = 0..3
let g = read<[R, C]Int>()
print(g[1, 0], g[0, 2])
