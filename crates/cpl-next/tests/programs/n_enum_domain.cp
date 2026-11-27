// expect: ok, output "4 2 1\n0 1 2 3\n3 3 1\n1 1\n2 YES 2"
// rule: R02.7 R04.2.16 R04.2.20 R04.2.22
// A payload-free enum is a domain by core's `Domain`: its positions are its variants in
// declaration order, so `.ord`, `D.nth`, `[D]T` tables, the walk (`D.find(p)`) and a
// `D: Domain` bound all work on it.
enum D = N | E | S | W
print(D.count, D.S.ord, D.nth(1).ord)
print([d.ord for d in D])
var a: [D]Int = 0
a[D.W] = 3
fn total<X: Domain>(x: X) -> Int = X.count + x.ord - 1
print(a[D.W], total(D.N) + a[D.N] - 0, Int(D.E))
print(Int(D.S) - D.S.ord + D.count(d => d != D.N) - 2, D.find(d => d != D.N)!.ord)
print(D.W.prev!.ord, yes(D.W.next == None), D.N.next!.next!.ord)
