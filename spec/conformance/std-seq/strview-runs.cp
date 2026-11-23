// expect: ok, output "1 b 2\n3 d 1"
// rule: 07.2.1
let ss = "abbd"
for r in ss[1..4].runs(): print(r.keys.first!, r.first!, r.len)
