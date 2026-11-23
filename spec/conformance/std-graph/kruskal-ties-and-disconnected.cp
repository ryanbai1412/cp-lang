// expect: ok, output "3\nYES NO YES NO"
domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V, w: Int) = [(u: V(0), v: V(1), w: 1), (u: V(1), v: V(0), w: 1), (u: V(2), v: V(3), w: 2), (u: V(0), v: V(1), w: 5)]
let (total, take) = kruskal(es)
print(total)
print(yes(take[E(0)]), yes(take[E(1)]), yes(take[E(2)]), yes(take[E(3)]))
