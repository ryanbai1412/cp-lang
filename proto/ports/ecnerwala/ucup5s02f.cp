fn covers(k: I128, v: I128, segs: List<(Int, Int)>, t: I128) -> Bool:
    var i: I128 = 0
    for (l, r) in segs:
        if i >= k: return true
        let lo = I128(6) * k * I128(l) - t * v
        let hi = I128(6) * k * I128(r) - t * v
        if lo > t * I128(6) * i: return false
        if hi >= t * I128(6) * i: i = hi / (t * I128(6)) + I128(1)
    i >= k

fn solve(t: I128, p0: List<(Int, Int)>, p1: List<(Int, Int)>) -> Int:
    var k: I128 = 1
    while I128(6) * k <= t:
        if not covers(k, I128(6), p0, t): return -1
        if not covers(k, I128(1), p1, t) or not covers(k, I128(5), p1, t): k *= I128(6)
        elif not covers(k, I128(2), p1, t) or not covers(k, I128(4), p1, t): k *= I128(3)
        elif not covers(k, I128(3), p1, t): k *= I128(2)
        else: return Int(I128(6) * k)
    -1

let (n0, n1): (Int, Int) = read()
let p0 = [read::<(Int, Int)>() for _ in 0..n0]
let p1 = [read::<(Int, Int)>() for _ in 0..n1]
let q: Int = read()
for _ in 0..q: print(solve(I128(read::<Int>()), p0, p1))
