type Id = 0..=100
var nodes: List<Int> = []
let id = nodes.push_id(7)
let (D, arr) = nodes.freeze()
print(id, D.count, arr[D.first])
