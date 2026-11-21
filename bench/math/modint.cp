// Matched benchmark: Mod<998244353> arithmetic (+, -, *, /, **) over 10M ops + 200K pow/inv.
type Z = Mod<998_244_353>

let n = 10_000_000
var a = Z(1234567)
var b = Z(89012345)
var acc = Z(0)
for i in 1..=n:
    let c = a * b + Z(i)
    a = b - c
    b = c + a
    acc = acc + b

var inv_acc = Z(0)
var x = Z(314159265)
for i in 1..=200_000:
    inv_acc = inv_acc + (x ** 17) / Z(i)
    x = x * Z(3) + Z(1)

print((acc + inv_acc).v)
