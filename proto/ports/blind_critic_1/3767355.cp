// Corpus submission 3767355: bandwidth-k Laplacian determinant via profile bitmask DP (Mod<65521>)
fn popcnt(x0: Int) -> Int:
    var x = x0
    var c = 0
    while x > 0:
        c += x & 1
        x = x >> 1
    c

let k: 1..=5 = read()
let n_in: 1..=1000 = read()
let n = n_in - 1

fn get_val(u: Int, v: Int) -> Int:
    if u < 0 or u >= n or v < 0 or v >= n:
        return 0
    if u == v:
        return min(k, u) + min(k, n - u)
    if abs(u - v) <= k:
        return -1
    0

fn next_mask(j: Int, ind: Int) -> Int:
    (j | (1 << ind)) >> 1

fn conv(ind: Int, rel: Int) -> Int:
    ind + rel - k

let m = 1 << (2 * k)
domain Row = 0..=n
domain Mask = 0..m

var dp: [Row, Mask]Mod<65521> = 0
dp[Row.at(n), Mask.at((1 << k) - 1)] = 1

for i in (0..n).rev:
    for ji in 0..m:
        if popcnt(ji) != k:
            continue
        if (ji & 1) == 0:
            var psum = 0
            for a in (1..=2 * k).rev:
                psum += (ji >> a) & 1
            let mul = if (psum & 1) == 1: -1 else: 1
            let coeff = Mod<65521>(mul * get_val(i, conv(i, 0)))
            dp[Row.at(i), Mask.at(ji)] = coeff * dp[Row.at(i + 1), Mask.at(next_mask(ji, 0))]
        else:
            var acc: Mod<65521> = 0
            var psum = 0
            for a in (0..=2 * k).rev:
                if ((ji >> a) & 1) == 0:
                    let mul = if (psum & 1) == 1: -1 else: 1
                    let coeff = Mod<65521>(mul * get_val(i, conv(i, a)))
                    acc += coeff * dp[Row.at(i + 1), Mask.at(next_mask(ji, a))]
                else:
                    psum += 1
            dp[Row.at(i), Mask.at(ji)] = acc

print(dp[Row.at(0), Mask.at((1 << k) - 1)])
