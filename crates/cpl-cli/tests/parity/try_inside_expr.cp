fn f(m: Map<Int, Int>, k: Int) -> ?Int:
    let v = k + m.get(k)? * 2
    some(v + m.get(v)?)
fn g(xs: List<?Int>) -> ?Int:
    var s = 0
    for x in xs: s += 1 + x?
    some(s)
fn h(a: ?Int, b: ?Int) -> ?Int = some(a? + b?)
fn side(x: Int) -> Int:
    print("side", x)
    x
fn order(a: ?Int) -> ?Int:
    some(side(1) + a? + side(2))
var m: Map<Int, Int> = Map()
m[1] = 2
m[5] = 10
print(f(m, 1) ?? -1, f(m, 2) ?? -1, f(m, 5) ?? -1)
print(g([some(1), some(2)]) ?? -1, g([some(1), none]) ?? -1, h(some(1), some(2)) ?? -1, h(none, some(1)) ?? -1, h(some(1), none) ?? -1)
print(order(some(5)) ?? -1, order(none) ?? -1)
fn walk(line: Str, on_foot: [0..26]?Int) -> ?Int:
    var total = 0
    for i in 0..line.len:
        total += i + on_foot[line[i].ord - 'a'.ord]?
    some(total)
var of: [0..26]?Int = none
of[0] = some(3)
of[1] = some(4)
print(walk("abba", of) ?? -1, walk("abc", of) ?? -1)
fn best(xs: List<?Int>) -> ?Int:
    var b = 0
    for i in 0..xs.len: b max= i + xs[i]?
    some(b)
print(best([some(1), some(5)]) ?? -1, best([some(1), none]) ?? -1)
