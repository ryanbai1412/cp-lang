// expect: ok, output "0 2\n2 3\n5 1"
// rule: 07.7.10
let a = [1, 1, 2, 2, 2, 1]
for r in a.runs(by: x => x): print(r.keys.first!, r.len)
