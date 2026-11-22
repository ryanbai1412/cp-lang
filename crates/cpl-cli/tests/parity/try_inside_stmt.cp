fn side(x: Int) -> Int:
    print("side", x)
    x
fn g(x: ?Int) -> ?(Int, Int):
    some((side(5), x? + 1))
fn h(xs: List<Int>, x: ?Int) -> ?Int:
    let c1 = side(7) + x?
    if c1 > 100: return some(-1)
    let c2 = side(8) + x?
    if c2 > 50: return some(-2)
    var best = 0
    best max= side(9) + x?
    let c3 = side(6) + x?
    assert c3 >= 0
    for i in 0..side(1) + x?: best += xs[i]
    match side(3) + x?:
        4 => best += 100
        _ => best += 1
    some(xs[side(0) + x?] + best)
print(g(none) ?? (-1, -1), g(some(1)) ?? (-1, -1))
print(h([1, 2, 3], some(1)) ?? -1, h([1], none) ?? -1)
struct P:
    a: Int
    b: Int
fn mk(x: ?Int, y: ?Int) -> ?P = some(P(a: side(1) + x?, b: y? * 2))
fn idx(xs: List<Int>, x: ?Int) -> ?Int:
    var ys = xs
    ys[0] = side(2) + x?
    ys.push(x? + 1)
    some(sum(ys) + min(x?, side(4)))
print(mk(some(1), some(2)) ?? P(-1, -1), mk(some(1), none) ?? P(-1, -1), mk(none, some(1)) ?? P(-1, -1))
print(idx([1, 2], some(1)) ?? -1, idx([1, 2], none) ?? -1)
fn show(x: ?Int) -> ?Int:
    print(side(1), x? + 1, "end")
    print(side(2))
    some(0)
print(show(some(1)) ?? -1, show(none) ?? -1)
