// Source: NOIP 2019 Senior P2 (Brackets on a Tree), https://www.luogu.com.cn/problem/P5658
// Problem: a tree of n <= 5e5 vertices rooted at 1 (parent f_u < u), each vertex holding '(' or
//   ')'. k_i counts the substrings (by position) of the root-to-i string that are balanced.
//   Print the xor of i * k_i over all i.
// Solution: walk down from the root. open[v] is the nearest '(' on the path to v still
//   unmatched. A ')' at v closes j = open[parent]: the balanced strings ending at v are j..v plus
//   one per balanced string ending at j's parent, and open[v] becomes open[j's parent]. k[v] is
//   k[parent] plus those. Everything comes from the parent and j's parent: one preorder, O(n).
// original: corpus/submissions/5127828.cpp
// Bounds: n: 1..=5e5

let n: 1..=5e5 = read()
domain V = 1..=n
// WISH TR9: `let s: [V]Char = read()`; the interpreter reads a `[V]Char` as n
// separate tokens, so the string is read as `Str` and indexed by position.
let s: Str = read()
var boss: [V]?V = None
for v in V:
    if v == V.first: continue
    let b: V = read()
    boss[v] = b
let t = Rooted.from_parents(boss)

var open: [V]?V = [V] _ => None
var ending: [V]Int = 0  // balanced substrings that end exactly at v
var k: [V]Int = 0
var ans = 0
for v in t.at:
    let above = t.parent(v)
    if let a = above:
        open[v] = open[a]
        k[v] = k[a]
    if s[v.ord] == '(':
        open[v] = v
    elif let j = open[v]:
        ending[v] = 1
        open[v] = None
        if let b = t.parent(j):
            ending[v] += ending[b]
            open[v] = open[b]
    k[v] += ending[v]
    ans ^= v.label * k[v]
print(ans)
