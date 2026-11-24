// expect: ok, output "0 1 2\n3\n7"
// rule: 07.12.10
struct Node(val: Int, next: ?Int)
var nodes: List<Node> = []
let a = nodes.push_id(Node(3, None))
let b = nodes.push_id(Node(4, Some(a)))
let c = nodes.push_id(Node(0, Some(b)))
print(a, b, c)
print(nodes.len)
var sum = 0
var at = nodes[c].next
while let Some(i) = at:
    sum += nodes[i].val
    at = nodes[i].next
print(sum)
