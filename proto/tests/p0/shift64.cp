let k: 0..=62 = read()
var mask = 1
mask <<= 33
print(1 << k, 1 << 32, mask >> 1 << 9, (1 << k) >> 37)
