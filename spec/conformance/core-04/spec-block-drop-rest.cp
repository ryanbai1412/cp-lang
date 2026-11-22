// expect: ok, output "1"
// rule: 04b.1.11
let a = [3, 1, 4, 1]
var budget = 5
var rest = a[..]
while let x = rest.first:
    if x > budget: break
    budget -= x
    rest = rest.drop(1)
print(budget)
