// expect: ok, output "3 3"
// rule: 07.1.6
// A `Prefix` built directly, not by `scan`, also reads a position as its right cut.
domain D = 0..3
let a: [D]Int = [D] v => 1 + Int(v)
let p = Prefix(a)
let q = Prefix<D, Int>(a)
print(p[D(1)], q[D(1)])
