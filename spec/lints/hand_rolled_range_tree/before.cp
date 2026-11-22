var bit: [0..=8]Int = 0
var p = 3
while p <= 8:
    bit[p] += 1
    p += p & -p
print(bit[4])
