// expect: ok, output "30\n30"
// rule: 04a.8.2 04a.9.2
let a = [10, 20, 30, 40]
let w = a[1..3].rev
print(w.nth(0))
print(a[w.keys.first!])
