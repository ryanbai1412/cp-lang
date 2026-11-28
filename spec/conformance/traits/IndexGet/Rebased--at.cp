// expect: ok, output "4\n1\n5"
// engines: next
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
for k in r.keys: print(r[k])
