// expect: ok, output "10"
// rule: 01.5.7
// The right side of `and`/`or` runs only when the left does not decide: no index trap.
let a = [1, 2]
let j = 2
var c = 0
if j < 2 and a[j] > 0: c += 1
if j >= 2 or a[j] > 0: c += 10
print(c)
