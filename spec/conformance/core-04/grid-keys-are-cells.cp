// expect: ok, output "0 2"
// rule: 04.2.10
// input: "..A\n...\n"
// A `[R, C]T` grid is `[(R, C)]T`: its keys are cell tuples, so `g[p]` is one cell (rebuild round 6 H-GRID-FIND).
domain R = 0..2
domain C = 0..3
let g = read<[R, C]Char>()
for p in g.keys:
    if g[p] == ('A'):
        print(Int(p.0), Int(p.1))
