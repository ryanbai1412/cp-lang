// let: k = \log n
let n: Int = read()
var acc = 0
let b = [i % 1000 for i in 0..n]
let c = [i % 997 for i in 0..n]
acc ^= convolve(b, c).len // timed: ntt.convolve
acc ^= convolve(c, b).len // timed: ntt.convolve
print(acc)
