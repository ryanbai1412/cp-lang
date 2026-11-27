// expect: ok, output "2\n3\n1 1\n2 2\n2 3\n3 4"
// rule: R07.1.1 R04.4.4
// A view of a domain-keyed array walks with `for`, its keys the source's positions.
domain V = 1..=5
let a = [V] v => Int(v)
for x in a[V.cuts(1)..V.cuts(3)]:
    print(x)
for (k, x) in a[..V.cuts(2)].items:
    print(Int(k), x)
for (k, x) in a[V.cuts(1)..V.cuts(3)].items:
    print(Int(k), x + 1)
