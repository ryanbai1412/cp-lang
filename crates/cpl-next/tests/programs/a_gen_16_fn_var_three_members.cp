// expect: ok, output "8"
// rule: decision 1
// 20 lambdas flowing into one var: the set grows, the call is a branch.
var f = (x: Int) => x
for i in 0..3:
    if i == 0: f = (x: Int) => x + 1
    if i == 1: f = (x: Int) => x * 2
    if i == 2: f = (x: Int) => x * x - 1
print(f(3))
