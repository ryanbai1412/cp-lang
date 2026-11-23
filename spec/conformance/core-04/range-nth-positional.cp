// expect: ok, output "2 3\nYES"
// rule: 04.4.32
let r = 2..5
print(r.nth(0), r.nth(1))
print(yes(r.nth(2) == r.last!))
