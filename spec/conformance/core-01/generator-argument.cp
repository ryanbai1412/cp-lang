// expect: ok, output "5 2"
// rule: 01.10.1
let xs = [1, 2, 3]
print(sum(x for x in xs if x > 1), count(x > 1 for x in xs))
