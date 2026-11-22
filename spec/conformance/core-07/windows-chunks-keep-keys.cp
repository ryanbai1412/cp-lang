// expect: ok, output "0\n2\n4\n3"
// rule: 07.7.9
let a = [1, 2, 3, 4, 5]
for w in a.chunks(2): print(w.keys.first!)
print(a.windows(3).count(w => true))
