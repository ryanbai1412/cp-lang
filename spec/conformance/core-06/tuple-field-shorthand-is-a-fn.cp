// expect: ok, output "1 2\n3 1\n1 2"
// rule: 06.1.4
// A tuple field alone is a fn in argument position, as a named field is: `by: .0` is `p => p.0`.
print([(3, 1), (1, 2)].min(by: .0)!)
print([(3, 1), (1, 2)].min(by: .1)!)
print([(3, 1), (1, 2)].max(by: .1)!)
