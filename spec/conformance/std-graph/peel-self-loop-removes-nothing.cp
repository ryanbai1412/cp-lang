// expect: ok, output "0"
domain V = 0..1
let g: [V]List<V> = [[0]]
print(peel(g).len)
