// expect: ok, output "0\n1\n2"
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
for k in r.keys: print(k)
