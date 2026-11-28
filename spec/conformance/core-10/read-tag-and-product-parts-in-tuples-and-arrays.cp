// expect: ok, output "2 2\n1 2 YES 5\n2 3 NO 9\n2\n1"
// input: "3\n2 first second second\n1 2 first 5\n2 3 second 9\nfirst 1 1 second 2 2 first 1 3 second 2 1\nsecond 1 1 second 1 2 second 2 2 first 2 3\n"
// rule: 10.2.3
// Reads whose parts read by tag or by parts (R10.2.3): an enum and a product
// position inside tuples and arrays, beside bounded numbers, read the same
// at top level, in a loop and in a fn.
enum Run:
    @tag("first") First
    @tag("second") Second
domain R = 1..=2
domain C = 1..=3
domain Cell = (R, C)
let k: 1..=4 = read()
let (n, runs): (1..=k, [k]Run) = read()
print(n, count(r == Second for r in runs))
let pairs: [2](Cell, Run, 0..=9) = read()
for (c, r, d) in pairs:
    print(c, yes(r == First), d)
fn grid(m: Int) -> [2, 2](Run, Cell):
    var t = 0
    for _ in 0..m:
        t += 1
    let g: [2, 2](Run, Cell) = read()
    g
for _ in 0..2:
    let g = grid(1)
    print(count(g[i, j].0 == First for i in 0..2, j in 0..2))
