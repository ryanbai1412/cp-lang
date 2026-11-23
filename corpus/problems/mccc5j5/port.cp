// Source: Mock CCC '23 Contest 1 J5 (Calculus Problem), https://dmoj.ca/problem/mccc5j5
// Problem: N <= 1e6 problems worth d_0..d_{N-1} points and Q <= 5e5 people (m_j < N,
//   k_j <= 1e6); person j can solve problem i iff m_j xor i <= k_j. For each person print the
//   most points of a problem they can solve.
// Solution: a max segment tree over 2^bits leaves (bits covers N - 1, every m and every k) is
//   a binary trie of the indices. Walk the path of i = m xor k from the root; wherever k has a
//   1 bit, the sibling subtree has m xor i < k on every leaf, so take its max; the leaf at the
//   end is m xor i = k itself. O(2^bits) build, O(bits) per query.
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
    hi max= m
    hi max= k

let bits = max(1, bit_width(hi))
let sz = 1 << bits
domain Node = 0..(2 * sz)
var tree: [Node]Int = [Node] _ => 0

for i in I:
    tree[Node(sz + i.ord)] = a[i]
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
            ans max= sib
        o = 2 * o + chi
    max(ans, tree[Node(o)])

for j in 0..q:
    print(query(qm[j], qk[j]))
