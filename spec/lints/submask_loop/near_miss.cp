let n: 1..=16 = read()
var bits = 0
var s = n
while s > 0:
    bits += 1
    s &= s - 1
print(bits)
