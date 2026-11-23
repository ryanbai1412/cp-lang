// expect: ok, output "0"
domain V = 0..2
let g: [V]List<V> = [[1], []]
let none_from: List<V> = []
print(dfs(g, from: none_from).collect().len)
