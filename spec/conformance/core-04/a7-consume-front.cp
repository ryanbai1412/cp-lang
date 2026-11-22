// expect: ok, output "0\n2\n1"
// rule: 04a.2.7 04a.9.1 04b.1.4
let a = [3, 1, 4, 1, 5]
var budget = 8
var rest = a[..]
while let x = rest.first:
    if x > budget: break
    budget -= x
    rest = rest.drop(1)
print(budget)
print(rest.len)
let k = 3
print(rest[k])
