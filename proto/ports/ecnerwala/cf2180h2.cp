fn solve_one(g_hi0: Int, d: Int) -> Int:
    if d % 2 == 1:
        let g_hi = g_hi0 / d + ((g_hi0 % d) & 1) + 1
        let msk = 0x5555555555555554
        let cnt = popcount(g_hi) - 2 * popcount(g_hi & msk)
        let tbl = [0, 2, 3, 1]
        tbl[cnt & 3]
    else:
        if g_hi0 % 2 == 0: return 0
        let g = g_hi0 / d + 2
        if ctz(g) % 2 == 1: 0
        elif popcount(g) % 2 == 1: 2
        else: 1

@!multitest
let n: Int = read()
var ans = 0
for _ in 0..n:
    let (a, b, c, l, r): (Int, Int, Int, Int, Int) = read()
    let d = c - b
    ans ^= solve_one(r - c + 1, d) ^ solve_one(l - c, d)
print(if ans != 0: "Bug" else: "Feature")
