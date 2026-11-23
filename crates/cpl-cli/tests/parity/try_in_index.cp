fn side(k: Int) -> Int:
    print("side", k)
    k
fn idx(xs: List<Int>, x: ?Int, y: ?Int) -> ?Int:
    var ys = xs.clone()
    ys[x?] = 5
    ys[x?] += side(1)
    ys[side(0)] = y? + side(2)
    var g: [0..2, 0..2]Int = 0
    g[x?, side(1)] = y?
    g[side(0), y? - 7] max= 3
    some(ys[0] + ys[1] + g[1, 1] + g[0, 0])
print(idx([1, 2], some(1), some(7)) ?? -1, idx([1, 2], none, some(7)) ?? -1, idx([1, 2], some(0), none) ?? -1)

fn bind(x: ?Int) -> ?Int:
    let v = x?
    var w = 1
    w = x? + v
    some(w)

print(bind(some(3)) ?? -1, bind(none) ?? -1)
