// expect: ok, output "23 15 5 1"
// rule: R06.1.8
// A declared fn passed where fewer parameters are wanted: its defaults fill the rest.
fn bump(x: Int, by: Int = 10) -> Int = x + by
fn wrap<T>(x: T, n: Int = 3, m: Int = 2) -> List<T>:
    var out = [x]
    for i in 1..n - m: out.push(x)
    out
fn scale(x: Int; by: Int = 3) -> Int = x * by
fn ap(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
fn sum(f: Int -> Int, xs: List<Int>) -> Int:
    var s = 0
    for x in xs: s += f(x)
    s
fn first(f: Int -> List<Int>) -> Int = f(7).len
print(sum(bump, [1, 2]), sum(scale, [2, 3]), ap(bump, 2, 3), first(wrap))
