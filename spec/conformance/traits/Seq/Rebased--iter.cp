// expect: ok, output "4\n1\n5"
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
for x in r: print(x)
