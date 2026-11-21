fn solve_one(g_hi0: Int, d: Int) -> Int:
    if d % 2 == 1:
        let g_hi = g_hi0 / d + ((g_hi0 % d) & 1) + 1
        let cnt = popcount(g_hi) - 2 * popcount(g_hi & 0x5555555555555554)
        [0, 2, 3, 1][cnt & 3]
    elif g_hi0 % 2 == 0:
        0
    else:
        let g = g_hi0 / d + 2
        if ctz(g) % 2 == 1: 0
        elif popcount(g) % 2 == 1: 2
        else: 1

@!multitest
let n: Int = read()
let qs: [0..n](Int, Int, Int, Int, Int) = read()
let ans = fold(^, solve_one(r - c + 1, c - b) ^ solve_one(l - c, c - b) for (_a, b, c, l, r) in qs)
print(if ans != 0: "Bug" else: "Feature")
