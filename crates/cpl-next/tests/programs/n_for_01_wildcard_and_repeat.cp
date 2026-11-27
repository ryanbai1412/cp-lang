// expect: ok, output "3\n2\n3"
// `for _ in xs` walks `xs` binding nothing, and `repeat n:` is `for _ in 0..n:`.
var n = 0
for _ in 0..3: n += 1
print(n)
var k = 0
repeat 2:
    k += 1
print(k)
print([0 for _ in [7, 8, 9]].len)
