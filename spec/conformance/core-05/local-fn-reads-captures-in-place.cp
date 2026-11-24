// expect: ok
// rule: 05.10.1
// tool: build -o -
// has: "auto at = [&g_"
// lacks: "cpl::Env<"
// A nested fn only called where it is made borrows what it captures at each call (R10), so it reads `g` in place instead of copying it into an env.
type Big = (a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int)
fn pick(g: Big, i: Int) -> Int:
    fn at(j: Int) -> Int = if j == 0: g.a else: g.h
    at(i) + at(1 - i)
let n: Int = read()
var s = 0
for i in 0..n:
    let g: Big = (i, 1, 2, 3, 4, 5, 6, 7)
    s += pick(g, i % 2)
print(s)
