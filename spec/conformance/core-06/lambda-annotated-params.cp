// expect: ok, output "12"
// rule: 06.2.3
// A lambda may annotate its parameters (rebuild round 19 H04).
let f = (s: Int, y: Int) => s * y
print(f(3, 4))
