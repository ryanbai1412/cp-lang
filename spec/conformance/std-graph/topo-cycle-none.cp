// expect: ok, output "NO\n0 2 1"
domain V = 0..3
let cyc: [V]List<V> = [[V(1)], [V(2)], [V(1)]]
print(yes(topo(cyc) != None))
let dag: [V]List<V> = [[V(2)], [], [V(1)]]
print(topo(dag)!)
