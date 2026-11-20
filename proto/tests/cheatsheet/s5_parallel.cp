// cp: 0.1 bea787c+dirty
var (r0, r1) = (240, 46)
while r1 != 0:
    let q = r0 / r1
    (r0, r1) = (r1, r0 - q * r1)
print(r0)
