// DMOPC '21 Contest 7 P2 — move a contiguous block l..r to just after k,
// on an intrusive doubly linked list with a head sentinel.
// original: corpus/submissions/4452915.cpp
let (n, q): (Int, Int) = read()
domain Node = 0..=(n + 1)             // 0 = head sentinel, n + 1 = tail sentinel
domain Q = 0..q

// FRICTION F3: no stdlib linked list; `?Node` links would need `!` at every
// splice, so the sentinels keep both arrays total instead.
var next: [Node]Node = [Node] v => v.next ?? v
var prev: [Node]Node = [Node] v => v.prev ?? v

fn link(a: Node, b: Node):
    next[a] = b
    prev[b] = a

for _ in Q:
    let (l, r, k): (Node, Node, Node) = read()
    if next[k] != l:
        let (after_k, before_l, after_r) = (next[k], prev[l], next[r])
        link(k, l)
        link(r, after_k)
        link(before_l, after_r)

var order: List<Node> = []
var cur = Node.first
for _ in 0..n:
    cur = next[cur]
    order.push(cur)
print(order)
