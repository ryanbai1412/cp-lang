fn solve_one(g_hi0: Int, d0: Int) -> Int:
    let l = ctz(d0)
    let d = d0 >> l
    var g_hi = g_hi0 / d + ((g_hi0 % d) & 1)
    var ans = Int(g_hi.bit(l + 1))
    g_hi &= (2 << l) - 1
    if g_hi == 0: return ans
    ans ^= 1
    if g_hi % 2 == 1:
        if g_hi.bit(l):
            let i = bit_width((2 << l) - g_hi) - 1
            ans ^= if ((l ^ i) & 1) == 1: 2 else: 1
        else:
            let i = bit_width(g_hi) - 1
            ans ^= if ((l ^ i) & 1) == 1: 0 else: 1
    ans

@!multitest
let n: Int = read()
var ans = 0
for _ in 0..n:
    let (a, b, c, l, r): (Int, Int, Int, Int, Int) = read()
    let d = c - b
    ans ^= solve_one(r - c + 1, d) ^ solve_one(l - c, d)
print(if ans != 0: "Bug" else: "Feature")
