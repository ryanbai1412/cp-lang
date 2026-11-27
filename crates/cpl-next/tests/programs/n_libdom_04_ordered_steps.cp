// expect: ok, output "4 2 0\n1 3"
// rule: R04.2.31
// A generic body that steps bounds `D: Ordered`, which every core domain meets: a range, an enum
// and a product.
enum Dir:
    N
    E
    S
domain R = 0..5
fn after<D: Ordered>(d: D) -> D = d.next ?? d
print(Int(after(R(3))), Int(after(Dir.E)), Int(after(R(4))) - 4)
let p = after((R(1), R(2)))
print(Int(p.0), Int(p.1))
