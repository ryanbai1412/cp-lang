// let: k = \log n
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let g2 = [[Z(r * n + j) for j in 0..n / 8] for r in 0..4]
for _ in 0..4:
    acc ^= convolve_2d(g2, g2).len // timed: ntt.convolve_2d
print(acc)
