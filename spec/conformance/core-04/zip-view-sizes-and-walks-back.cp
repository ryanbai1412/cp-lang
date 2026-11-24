// expect: ok, output "3\n6\n43 32 21"
// rule: 04b.1.3
// A zip is as long as its shorter side, keyed by its first, and walks back from that length.
let a = [10, 20, 30, 40]
let b = [1, 2, 3]
let z = zip(a[1..], b)
print(z.len)
var s = 0
for k in z.keys: s += k
print(s)
print(" ".join(["{x + y}" for (x, y) in z.rev]))
