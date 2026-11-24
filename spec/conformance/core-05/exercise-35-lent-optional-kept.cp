// expect: ok, output "0"
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let queries = List((V.first!, V.last!))
var best: ?[V]Int = None
for (l2, r2) in queries:
    best = a[l2..r2]
print(best!.sum())
