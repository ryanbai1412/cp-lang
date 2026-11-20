let (a, b): (Int, Int) = read()
let p: I128 = I128(a) * b
print(p)
var acc: I128 = 0
for i in 0..5:
    acc += p * i
acc -= 1
print(acc, acc / 7, acc % 7, -acc / 7, -acc % 7)
print(Int(p % 1_000_000_007))
print(p > a, p == I128(a) * b, max(p, acc), min(p, acc), abs(-p))
let big: I128 = read()
print(big, big * big, big >> 3, abs(big) << 2)
var h: U64 = 1469598103934665603
for c in "hello":
    h = (h ^ U64(Int(c))) * 1099511628211
print(h)
let m: U64 = 1 << 63
print(m, m >> 62, m + m, ~m, m | 5, U64(-1), F64(m))
let u: U64 = read()
print(u, u / 3, u % 10, u > m)
var xs: List<I128> = []
xs.push(p)
xs.push(3)
print(xs, sum(xs))
print((p / 1e9) ** 2, h ** 3)
