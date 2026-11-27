// expect: ok, output "-15 -5"
// An inlined fn's result goes straight to where the caller keeps it, but not when that is
// a local the fn's own parameters read: then it is built first and moved after.
fn pick(a: Int, b: Int) -> Int:
    if a > b:
        return b - a
    return a * b + a
var x = 5
var y = 2
x = pick(x, y)
y = pick(y, x)
x = pick(x, 4)
print(x, y)
