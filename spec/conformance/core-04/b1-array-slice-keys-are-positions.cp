// expect: ok, output "2 2\n5 5\n1 1\n3 4 5\n4"
// rule: 04b.1.2
domain V = 1..=5
let b = [V] v => Int(v) * 3 % 7
let w = b[V(3)..=V(5)]
for (i, x) in w.items: print(b[i], x)
print(w.keys.map(k => Int(k)).collect())
print(Int(w.argmax()!))
