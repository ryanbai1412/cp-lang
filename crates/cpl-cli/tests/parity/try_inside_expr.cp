fn f(m: Map<Int, Int>, k: Int) -> ?Int:
    let v = k + m.get(k)? * 2
    Some(v + m.get(v)?)
fn g(xs: List<?Int>) -> ?Int:
    var s = 0
    for x in xs: s += 1 + x?
    Some(s)
fn h(a: ?Int, b: ?Int) -> ?Int = Some(a? + b?)
fn side(x: Int) -> Int:
    print("side", x)
    x
fn order(a: ?Int) -> ?Int:
    Some(side(1) + a? + side(2))
var m: Map<Int, Int> = Map()
m[1] = 2
m[5] = 10
print(f(m, 1) ?? -1, f(m, 2) ?? -1, f(m, 5) ?? -1)
print(g([Some(1), Some(2)]) ?? -1, g([Some(1), None]) ?? -1, h(Some(1), Some(2)) ?? -1, h(None, Some(1)) ?? -1, h(Some(1), None) ?? -1)
print(order(Some(5)) ?? -1, order(None) ?? -1)
fn walk(line: Str, on_foot: [0..26]?Int) -> ?Int:
    var total = 0
    for i in 0..line.len:
        total += i + on_foot[line[i].ord - 'a'.ord]?
    Some(total)
var of: [0..26]?Int = None
of[0] = Some(3)
of[1] = Some(4)
print(walk("abba", of) ?? -1, walk("abc", of) ?? -1)
fn best(xs: List<?Int>) -> ?Int:
    var b = 0
    for i in 0..xs.len: b max= i + xs[i]?
    Some(b)
print(best([Some(1), Some(5)]) ?? -1, best([Some(1), None]) ?? -1)
