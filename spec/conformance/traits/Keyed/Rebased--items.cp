// expect: ok, output "0 4\n1 1\n2 5"
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
for (k, x) in r.items: print(k, x)
