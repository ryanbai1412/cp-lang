// expect: ok, output "45 3 9"
// input: "5\n"
// rule: 05.10.1
// A nested fn only called in its home reads its capture in place; one that escapes keeps the value it captured, even after the source changes.
type Big = (a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int)
fn pick(g: Big, i: Int) -> Int:
    fn at(j: Int) -> Int = if j == 0: g.a else: g.h
    at(i) + at(1 - i)
fn keep(g: Big) -> (Int -> Int):
    fn at(j: Int) -> Int = if j == 0: g.a else: g.h
    at
let n: Int = read()
var s = 0
for i in 0..n:
    let g: Big = (i, 1, 2, 3, 4, 5, 6, 7)
    s += pick(g, i % 2)
var g: Big = (3, 1, 2, 3, 4, 5, 6, 9)
let f = keep(g)
g.a = 100
var h = 0
fn last(j: Int) -> Int = if j == 0: g.a else: g.h + h
h = 0
print(s, f(0), last(1))
