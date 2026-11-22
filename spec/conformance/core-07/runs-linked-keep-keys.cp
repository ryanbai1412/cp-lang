// expect: ok, output "1 2\n3 3"
// rule: 07.7.10
let a = [9, 1, 2, 4, 5, 6]
for r in a[1..].runs_linked((x, y) => y == x + 1): print(r.keys.first!, r.len)
