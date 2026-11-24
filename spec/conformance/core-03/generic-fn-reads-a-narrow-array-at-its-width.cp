// expect: ok, output "1000000000 7 1000000000"
// rule: 03.1.1
// input: "3\n1000000000 7 1000000000\n"
// A narrow array passed to a generic fn and walked by a std seq body keeps its values: the instance takes the caller's width.
let n: 2..=100 = read()
let a: [0..n]1..=10 ** 9 = read()
fn f<A: Positional<Int>>(a: A) -> Int = a.nth(0)
var worst = 0
for (x, y) in a.adjacent: worst = max(worst, y - x)
print(f(a), a[1], worst + 7)
