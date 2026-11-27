// expect: ok, output "5 0 4\n2 3 1 2\n1 2\n31 23 4\n2 10\n3 5"
// rule: R04.2.5 R04.2.16 R04.2.17 R04.3.1 R04.3.2 R04.4.4
// `V.cuts` is core's `@member(cuts)` struct `Cut<V>`, a domain of `V.count + 1` cuts; `v.l`, `v.r`,
// `c.l`, `c.r`, `.next(k)` and `.prev(k)` are `Domain`'s and `Cut`'s getters, and `j..i` of cuts
// indexes the positions between them.
domain V = 1..=4
fn cuts_of<D: Domain>(x: D) -> Int = Int(x.r) - Int(x.l)
let c: V.cuts = V.cuts(2)
print(V.cuts.count, Int(V.cuts.first!), Int(V.cuts.last!))
print(Int(c), Int(V(3).r), Int(V(2).l), cuts_of(V(1)) + 1)
print(Int(V.cuts.at(1) ?? V.cuts(0)), Int(V.cuts.at(9) ?? V.cuts(2)))
print(Int(c.next!) * 10 + Int(c.prev!), Int(c.l!) * 10 + Int(c.r!), Int(V.cuts(0).l ?? V(4)))
let a = [V] v => Int(v) + 1
let w = a[V.cuts(1)..V.cuts(3)]
print(w.len, Int(V(1).next(3)!) + Int(V(4).prev(2)!) + Int(c.next(2)!))
print((V.cuts(1)..=V.cuts(3)).len, Int(V(1).r.r!) + Int(V(2).next ?? V(1)))
