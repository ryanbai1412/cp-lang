@!multitest
let x: 0..=1000000000000000000 = read()
var r = Int(sqrt(F64(x)).floor)  // expect: missing float_sqrt_for_int
while r * r > x:
    r -= 1
while (r + 1) * (r + 1) <= x:
    r += 1
print(r)
