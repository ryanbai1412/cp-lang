// expect: ok, output "4\n3\n2\n3 1"
// rule: 07.5.1 07.5.5 07.5.6 07.5.7
let a = [1, 2, 3, 4, 5]
let w = a[1..4]
for x in w.rev: print(x)
print(w.len, w.keys.first!)
