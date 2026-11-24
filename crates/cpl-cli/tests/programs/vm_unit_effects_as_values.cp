// `sort` and `print` as values: closure bodies and branches
// of an `if` expression run them as their own VM ops.
fn later(a: Int, b: Int) -> Bool = a > b

var xs = [3, 1, 4, 1, 5]
let sort_down = () => xs.sort(cmp: later)
sort_down()
print(xs)
xs.sort(cmp: (a, b) => a < b)
print(xs)
let show = (x: Int) => print(x, x * x)
show(7)
let report = (n: Int) => if n > 3: print("long", n) else: print("short", n)
report(xs.len)
report(2)
