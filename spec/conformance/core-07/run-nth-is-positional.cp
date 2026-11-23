// expect: ok, output "1 2 0\n2 3 2"
// rule: 07.4.5
// `.nth(k)` reads by walk position on any view; `.keys` are the parent's (decision 388).
let a = [1, 1, 2, 2, 2]
for r in a.runs():
    print(r.nth(0), r.len, r.keys.first!)
