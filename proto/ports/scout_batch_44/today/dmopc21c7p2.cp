// Runnable port (proto/cp.py lib-run) of dmopc21c7p2.cp
// original: corpus/submissions/4452915.cpp

let (n, q): (Int, Int) = read()
domain Q = 0..q
let ops: [Q](l: Int, r: Int, k: Int) = read()

// Node 0 is the head sentinel; `top + 1` is the first node no splice can move.
var top = n
for i in Q: top = max(top, max(ops[i].l, max(ops[i].r, ops[i].k)))
domain Node = 0..=(top + 1)
var next: [Node]Int = [Node] v => v.to_idx + 1
var prev: [Node]Int = [Node] v => v.to_idx - 1

fn link(a: Int, b: Int):
    next[Node(a)] = b
    prev[Node(b)] = a

for i in Q:
    let (l, r, k) = (ops[i].l, ops[i].r, ops[i].k)
    if next[Node(k)] != l:
        let after_k = next[Node(k)]
        let before_l = prev[Node(l)]
        let after_r = next[Node(r)]
        link(k, l)
        link(r, after_k)
        link(before_l, after_r)

var line = ""
var cur = 0
for i in 0..n:
    cur = next[Node(cur)]
    if i > 0: line = line + " "
    line += "{cur}"
print(line)
