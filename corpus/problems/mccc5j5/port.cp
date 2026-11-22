// Runnable port (proto/cp.py lib-run) of mccc5j5.cp
// original: corpus/submissions/5333196.cpp

let (n, q): (Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()

var qm: List<Int> = []
var qk: List<Int> = []
var hi = n - 1
for _ in 0..q:
    let (m, k): (Int, Int) = read()
    qm.push(m)
    qk.push(k)
    if m > hi: hi = m
    if k > hi: hi = k

let bits = max(1, bit_width(hi))
let sz = 1 << bits
domain Node = 0..(2 * sz)
var tree: [Node]Int = [Node] _ => 0

for i in I:
    tree[Node(sz + i.to_idx)] = a[i]
for o in (1..sz).rev:
    tree[Node(o)] = max(tree[Node(2 * o)], tree[Node(2 * o + 1)])

fn query(m: Int, k: Int) -> Int:
    var o = 1
    var ans = 0
    for d in (0..bits).rev:
        let bit_m = (m >> d) & 1
        let bit_k = (k >> d) & 1
        let chi = bit_m ^ bit_k
        if bit_k == 1:
            let sib = tree[Node(2 * o + (1 - chi))]
            if sib > ans: ans = sib
        o = 2 * o + chi
    max(ans, tree[Node(o)])

for j in 0..q:
    print(query(qm[j], qk[j]))
