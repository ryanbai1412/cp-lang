// expect: R0203
// input: "0"
// A trap in a loop body a generator's yield runs reports the body's line, not the yield's.
fn walk(n: Int) -> yields Int:
    for i in 0..n:
        yield i
let k: Int = read()
var s = 0
for v in walk(3):
    s += 10 / (k + v)  // <- here
print(s)
