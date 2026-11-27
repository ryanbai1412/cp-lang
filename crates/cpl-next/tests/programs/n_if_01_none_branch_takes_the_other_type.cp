// expect: ok, output "1 2\n0\n3\n7\n9"
// With no expected type, an `if` whose `else` is a bare `None` takes its type from the first
// branch that gives a value (R01.7.1); a `None` holding no list is copied and dropped soundly.
let xs: ?List<Int> = [1, 2]
var left = 1
while let y = (if left > 0: xs else: None):
    left -= 1
    print(y)
let x: ?Int = Some(3)
var n = 0
for k in 0..2:
    let z = if k == 0: None elif k == 1: x else: None
    if let v = z: n += v
print(0)
print(n)
let half = (a: Int) => if a > 1: Some(a * 2) else: None
print((half(3) ?? 0) + 1)
print(if let h = half(1): h else: 9)
