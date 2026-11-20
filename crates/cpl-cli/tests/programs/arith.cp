// Integer arithmetic: `/` and `%` are Euclidean, so a remainder is
// never negative whatever the signs.
let n: Int = read()
for k in 0..n:
    let a: Int = read()
    let b: Int = read()
    print(a / b, a % b, a + b, a - b, a * b)
    assert a == (a / b) * b + a % b
    assert a % b >= 0
var x = 17
x /= 5
x %= 2
x += 40
x -= 1
x *= 3
print(x, -x / 7, -x % 7, x / -7, x % -7)
// A dividend the checker knows is not negative, a divisor it knows is
// positive: the C++ uses its own operators.
let p: 0..=1000 = read()
let q: 1..=1000 = read()
print(p / q, p % q, (p + 3) / 2, p % 1000000007)
print(2 ** 10, 3 ** 0, (-2) ** 3, 7 ** 2 % 5)
print(1 << 40, 1 << 3 >> 1, 6 & 3, 6 | 3, 6 ^ 3, ~5)
print(min(3, 9), max(3, 9), min(4, 2, 8), max(4, 2, 8), abs(-12), abs(12))
print(9223372036854775807, -9223372036854775807 - 1)
print(1000000 * 1000000, 3000000000 + 1)
