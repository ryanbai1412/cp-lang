// expect: ok, output "3 4 4 -3"
// rule: 02.11.1
let x = F64(7) / F64(2)
let y: Int = floor(x)
print(y, ceil(x), round(x), trunc(-x))
