// expect: ok, output "0"
domain V = 0..2
let g: [V]List<V> = [[1], [0]]
let (domain C, comp) = scc(g)
print(condense(g, comp)[comp[0]].len)
