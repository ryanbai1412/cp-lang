// expect: ok, output "1 1 1\n2 3 2\n5 1 1"
// rule: 07.7.10
let a = [1, 1, 2, 2, 2, 1]
for r in a[1..].runs(): print(r.keys.first!, r.len, r[r.keys.first!])
