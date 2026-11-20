struct Node:
    val: Int
    nxt: Int

var ends = [false]
fn step() -> Int:
    let k = ends.len
    ends.push(false)
    return k

// Index expression grows `ends` before LHS bounds check & write
for _ in 0..40:
    ends[step()] = true

// RHS grows `nodes` (forcing vector reallocation) before LHS slot capture
var nodes: List<Node> = [Node(val: 1, nxt: 0)]
fn add_node(v: Int) -> Int:
    for _ in 0..32:
        nodes.push(Node(val: v, nxt: 0))
    return nodes.len - 1

nodes[0].nxt = add_node(7)
nodes[0].val += add_node(9)
let b1 = if ends[1]: 1 else: 0
let b2 = if ends[40]: 1 else: 0
print(ends.len, b1, b2, nodes.len, nodes[0].nxt, nodes[0].val)
