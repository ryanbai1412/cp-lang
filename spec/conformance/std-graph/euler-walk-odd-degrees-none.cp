// expect: ok, output "YES"
domain V = 0..4
domain E = 0..2
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(2), v: V(3))]
print(yes(euler_walk(es, directed: false) == None))
