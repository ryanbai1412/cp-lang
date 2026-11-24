// expect: ok, output "1"
let a = [3, 1, 4, 1, 5]
let r = a[2..].rebase
print(r.nth(1))
