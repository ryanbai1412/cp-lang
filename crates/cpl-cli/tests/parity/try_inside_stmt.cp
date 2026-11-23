fn side(x: Int) -> Int:
    print("side", x)
    x
fn g(x: ?Int) -> ?(Int, Int):
    Some((side(5), x? + 1))
fn h(xs: List<Int>, x: ?Int) -> ?Int:
    let c1 = side(7) + x?
    if c1 > 100: return Some(-1)
    let c2 = side(8) + x?
    if c2 > 50: return Some(-2)
    var best = 0
    best max= side(9) + x?
    let c3 = side(6) + x?
    assert c3 >= 0
    for i in 0..side(1) + x?: best += xs[i]
    match side(3) + x?:
        4 => best += 100
        _ => best += 1
    Some(xs[side(0) + x?] + best)
print(g(None) ?? (-1, -1), g(Some(1)) ?? (-1, -1))
print(h([1, 2, 3], Some(1)) ?? -1, h([1], None) ?? -1)
struct P:
    a: Int
    b: Int
fn mk(x: ?Int, y: ?Int) -> ?P = Some(P(a: side(1) + x?, b: y? * 2))
fn idx(xs: List<Int>, x: ?Int) -> ?Int:
    var ys = xs
    ys[0] = side(2) + x?
    ys.push(x? + 1)
    Some(sum(ys) + min(x?, side(4)))
print(mk(Some(1), Some(2)) ?? P(-1, -1), mk(Some(1), None) ?? P(-1, -1), mk(None, Some(1)) ?? P(-1, -1))
print(idx([1, 2], Some(1)) ?? -1, idx([1, 2], None) ?? -1)
fn show(x: ?Int) -> ?Int:
    print(side(1), x? + 1, "end")
    print(side(2))
    Some(0)
print(show(Some(1)) ?? -1, show(None) ?? -1)
