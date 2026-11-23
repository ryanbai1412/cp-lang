// expect: ok, output "30 50 10\n20 40\n3 30"
// rule: R04a.9.3
// A step view keeps its source's keys: `at(i)` reads the source's key `i`, and `even` is `step(2)`.
let a = [10, 20, 30, 40, 50]
let e = a.step(2)
print(e.at(2), e.at(4), a.even.at(0))
let w = a[1..]
print(w.step(2).at(1), w.even.at(3))
var n = 0
for x in a.even: n += 1
print(n, a.even.nth(1))
