// expect: ok, output "6\n7\n7\n2"
// rule: 04b.1.4 04b.5.9
let w = [5, 6, 7, 8][1..3]
print(w.first!)
print(w.last!)
print(w.nth(1))
print(w.keys.nth(1))
