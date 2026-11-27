// expect: ok, output "181 4\n3"
// rule: R04.2.8
// rule: R04.2.31
// A type parameter bounded by `Ordered` has `Domain`'s fns (`D.count`, `D.nth`) and `Ordered`'s
// (`D.at`), chosen per instance; one whose body names them is a domain without the bound (R04.4.30).
domain V = 3..7
fn walk<D: Ordered>(x: D) -> Int:
    var s = 0
    for k in 0..D.count: s += Int(D.nth(k))
    let a = D.at(5)
    s * 10 + (if a == None: 0 else: 1)
struct S<D>(base: Int):
    fn at(self, i: Int) -> D = D.nth(base + i)
print(walk(V(4)), V.count)
print(Int(S<0..10>(1).at(2)))
