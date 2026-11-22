const MOD = 1_000_000_007
let n = 10
var x = 5
x &= 7; x |= 8; x ^= 1; x <<= 2; x >>= 1
print(popcount(x), ctz(x), clz(x), bit_len(x), ~x, x & 3, x | 1, x ^ 2, MOD % n, 1 << 40, 2 ** 10)
print(-7 / 2, -7 % 2, 7 / -2, 7 % -2, n == 10 and not (x < 0) or false, x != 3, x <= 3, x >= 3, x > 3)
var y = 3
y += 1; y -= 2; y *= 5; y /= 3; y %= 2; y min= 0; y max= 4; y gcd= 6
print(y, 1e18, true, 'a')   // line comment
