let k: 0..=62 = read()
let one = 1
var sh = 1
sh <<= 32
print(1 << k, sh, 7 / 3, 7 % 3, -7 / 2, -7 % 3, 2 ** 3, one << 10)
var b = 6
b &= 7
var c = 5
c |= 2
var d = 6
d ^= 3
var e = 64
e >>= 2
print(b, c, d, ~0, e, popcount(11), ctz(8), clz(1) - 61, clz(4), bit_len(5))
var x = 9
x min= 3
var y = 1
y max= 4
var z = 4
z gcd= 6
z lcm= 4
print(x, y, z, if true and not false or false: 2 else: 3)
