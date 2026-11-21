// `a..=b` up to `Int.MAX` stops there; up to a length it is a plain loop.
let n: Int = read()
for i in n - 2..=n:
    print(i)
let xs = [4, 5, 6]
var s = 0
for i in 0..=xs.len - 1:
    s += xs[i]
print(s)
var t = 0
for i in 0..=xs.len:
    t += i
print(t)
