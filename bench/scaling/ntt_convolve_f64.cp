// let: k = \log n
let n: Int = read()
var acc = 0
let fb = [F64(i % 1000) / 3.0 for i in 0..n]
let fc = [F64(i % 997) / 7.0 for i in 0..n]
acc ^= convolve(fb, fc).len // timed: ntt.convolve
print(acc)
