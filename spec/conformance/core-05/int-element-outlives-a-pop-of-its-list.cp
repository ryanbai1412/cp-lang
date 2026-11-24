// expect: ok, output "1 1"
// rule: 05.2.1 05.3.5
// An Int read from a list in a table holds no lend, so it can be pushed elsewhere and kept past a pop of its list.
var adj: [0..3]List<Int> = []
adj[0].push(1)
var order: List<Int> = []
for j in 0..adj[0].len:
    let v = adj[0][j]
    if v != 5:
        order.push(v)
let tx = adj[0][0]
adj[0].pop()
print(order.len, tx)
