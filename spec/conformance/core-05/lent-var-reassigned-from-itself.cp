// expect: ok, output "1"
// rule: 05.7.8
// A lent `var` may be reassigned a narrower piece of the same source: it is still declared after `a`, inside its block.
let a = [1, 2, 3]
var budget = 4
if true:
    var rest = a[..]
    while let x = rest.first:
        if x > budget: break
        budget -= x
        rest = rest.drop(1)
print(budget)
