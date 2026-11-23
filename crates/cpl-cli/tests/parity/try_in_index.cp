fn side(k: Int) -> Int:
    print("side", k)
    k
fn idx(xs: List<Int>, x: ?Int, y: ?Int) -> ?Int:
    var ys = xs
    ys[x?] = 5
    ys[x?] += side(1)
    let at = side(0)
    let yv = y else: return None
    ys[at] = yv + side(2)
    var g: [0..2, 0..2]Int = 0
    g[x?, side(1)] = yv
    g[side(0), y? - 7] max= 3
    Some(ys[0] + ys[1] + g[1, 1] + g[0, 0])
print(idx([1, 2], Some(1), Some(7)) ?? -1, idx([1, 2], None, Some(7)) ?? -1, idx([1, 2], Some(0), None) ?? -1)

fn bind(x: ?Int) -> ?Int:
    let v = x else: return None
    var w = 1
    w = (x? + v) ?? -1
    Some(w)

print(bind(Some(3)) ?? -1, bind(None) ?? -1)
