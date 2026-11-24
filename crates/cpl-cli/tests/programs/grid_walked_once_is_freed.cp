// A grid of `main` walked once, and read by nothing after, is freed after that walk:
// `cpl::Grid` has the `swap` every heap-backed runtime container has.
let h: 1..=10 = read()
let w: 1..=10 = read()
domain Row = 0..h
domain Col = 0..w
let g: [Row, Col]Char = read()
let walls = [c == '#' for c in g]
var k = 0
for b in walls if b: k += 1
print(k)
