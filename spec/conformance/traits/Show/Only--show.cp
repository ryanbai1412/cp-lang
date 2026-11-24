// expect: ok, output "0->2"
domain V = 0..3
let g: [V]List<V> = [[1, 2], [2], []]
print(g.only(keep: v => v != 1))
