use std.window.{MonoDeque}
let (n, k): (1..=5e6, 1..=5e6) = read()
let a: [0..n]0..=1e9 = read()
var q = MonoDeque<Int>.new(min)
var total = 0
var i = 0
for x in a:
    q.push(i, x)
    q.drop_before(i - k + 1)
    total += q.best!
    i += 1
print(total)
