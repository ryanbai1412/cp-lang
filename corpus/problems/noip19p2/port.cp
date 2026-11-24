// Source: NOIP 2019 Senior P2 (Brackets on a Tree), https://www.luogu.com.cn/problem/P5658
// Problem: a tree of n <= 5e5 vertices rooted at 1 (parent f_u < u), each vertex holding '(' or
//   ')'. k_i counts the substrings (by position) of the root-to-i string that are balanced.
//   Print the xor of i * k_i over all i.
// Solution: walk down from the root. open[v] is the nearest '(' on the path to v still
//   unmatched. A ')' at v closes j = open[parent]: the balanced strings ending at v are j..v plus
//   one per balanced string ending at j's parent, and open[v] becomes open[j's parent]. k[v] is
//   k[parent] plus those. Everything comes from the parent and j's parent: one preorder, O(n).
// original: corpus/submissions/5127828.cpp
let n = read<1..=5e5>()
domain V = 1..=n
domain Child = 2..=n
let s = read<[V]Char>()
let boss = read<[Child]V>()
let t = Rooted.from_parents([V] v => if v.label >= 2: boss[Child(v.label)] else: None)

var open: [V]?V = None   // the nearest '(' on the path to v still unmatched
var ending: [V]Int = 0   // balanced substrings that end exactly at v
var k: [V]Int = 0
for v in t.at:
    let up = t.parent(v)
    open[v] = up.and_then(a => open[a])
    if s[v] == '(': open[v] = v
    elif let j = open[v]:
        let before = t.parent(j)
        ending[v] = 1 + (ending[before?] ?? 0)
        open[v] = before.and_then(b => open[b])
    k[v] = (k[up?] ?? 0) + ending[v]
print(([V] v => v.label * k[v]).fold(^))
