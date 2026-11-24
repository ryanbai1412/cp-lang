// let: k = \log n
type W = Mod<1_000_000_007>
let n: Int = read()
var acc = 0
let wb = [W(i * 7919 + 3) for i in 0..n]
let wc = [W(1_000_000_006 - i) for i in 0..n]
acc ^= convolve(wb, wc)[n - 1].v // timed: ntt.convolve
print(acc)
