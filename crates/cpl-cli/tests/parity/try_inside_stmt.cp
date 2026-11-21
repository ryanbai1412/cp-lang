fn side(x: Int) -> Int:
    print("side", x)
    x
fn g(x: ?Int) -> ?(Int, Int):
    some((side(5), x? + 1))
fn h(xs: List<Int>, x: ?Int) -> ?Int:
    if side(7) + x? > 100: return some(-1)
    elif side(8) + x? > 50: return some(-2)
    var best = 0
    best max= side(9) + x?
    assert side(6) + x? >= 0
    for i in 0..side(1) + x?: best += xs[i]
    match side(3) + x?:
        4 => best += 100
        _ => best += 1
    some(xs[side(0) + x?] + best)
print(g(none), g(some(1)))
print(h([1, 2, 3], some(1)), h([1], none))
struct P:
    a: Int
    b: Int
fn mk(x: ?Int, y: ?Int) -> ?P = some(P(a: side(1) + x?, b: y? * 2))
fn idx(xs: List<Int>, x: ?Int) -> ?Int:
    var ys = xs
    ys[0] = side(2) + x?
    ys.push(x? + 1)
    some(sum(ys) + min(x?, side(4)))
print(mk(some(1), some(2)), mk(some(1), none), mk(none, some(1)))
print(idx([1, 2], some(1)), idx([1, 2], none))
fn show(x: ?Int) -> ?Int:
    print(side(1), x? + 1, "end")
    print(side(2))
    some(0)
print(show(some(1)), show(none))
