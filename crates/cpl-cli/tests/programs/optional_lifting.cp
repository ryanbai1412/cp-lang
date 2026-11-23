let a: ?Int = 10
let b: ?Int = 3
let n: ?Int = None

print((a + b) ?? -1)
print((a - b) ?? -1)
print((a * b) ?? -1)
print((a / b) ?? -1)
print((a % b) ?? -1)
print((a ** 2) ?? -1)
print((a & 6) ?? -1)
print((a | 5) ?? -1)
print((a ^ 6) ?? -1)
print((b << 2) ?? -1)
print((a >> 1) ?? -1)
print((-a) ?? -1)
print((~b) ?? -1)

print((a * n) ?? -1)
print((n / b) ?? -1)
print((a % n) ?? -1)
print((n & 6) ?? -1)
print((a | n) ?? -1)
print((a ^ n) ?? -1)
print((n << 2) ?? -1)
print((a >> n) ?? -1)
print((~n) ?? -1)

let x: ?F64 = 6.0
let y: ?F64 = 2.0
let nf: ?F64 = None
print((x * y) ?? -1.0)
print((x / y) ?? -1.0)
print((x * nf) ?? -1.0)

var best: ?Int = None
best min= a + b
best min= a * b
best max= a * b
print(best ?? -1)
