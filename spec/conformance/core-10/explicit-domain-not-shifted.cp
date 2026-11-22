// expect: ok, output "3 3"
// rule: 10.2.2
with io(base: 1)
domain V = 1..=5
let v = "3".parse<V>()!
print(v, v)
