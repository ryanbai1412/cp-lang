use std.seq.{prod, rotate_left, rotate_right, rotated_left, rotated_right, suffix_scan}

let xs = [3, 1, 4, 1, 5]
print(xs.scan(+))
print(xs.suffix_scan(max))
print(xs.rev.scan(max).rev)
print(xs.prod())

domain D = 0..4
let a: [D]Int = [4, 2, 5, 1]
print(a.scan(min))
print(a.suffix_scan(+))
print(a.rotated_left(1))
print(xs.rotated_left(2))
print(xs.rotated_right(2))
print("abcd".rotated_left(1))

var m = [1, 2, 3, 4]
m.rotate_left(1)
print(m)
m.rotate_right(2)
print(m)
