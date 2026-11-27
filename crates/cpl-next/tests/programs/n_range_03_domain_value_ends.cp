// expect: ok, output "3 9\n6\n4\n6\n8"
// A range whose ends are values of a declared domain, or of a type parameter bound by `Domain`, walks their positions.
domain V = 1..=5

fn firsts<D: Domain>(a: D, b: D) -> Int = (a..=b).count()

let r = V(3)..=V(5)
print(Int(r.first!), r.count() * 3)
print(firsts(V(1), V(5)) + 1)
for v in V(2)..V(5): print(Int(v) * 2)
