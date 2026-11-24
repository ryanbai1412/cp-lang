// expect: ok, output "5\n1\n4"
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
for x in r.rev: print(x)
