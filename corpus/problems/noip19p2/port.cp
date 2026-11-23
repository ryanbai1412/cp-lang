// NOIP '19 P2 - Brackets on a Tree: every vertex of a tree rooted at 1 holds
// '(' or ')'. Let k[v] be the number of balanced substrings of the string
// read from the root down to v. Print the xor of v * k[v] over all v.
// Walk down from the root. open[v] is the nearest ancestor-or-self '(' still
// unmatched on the way to v. A ')' at v closes j = open[parent]; the
// balanced strings ending at v are the one from j to v plus every balanced
// string ending just above j, and what is still open is whatever was open
// above j. Everything a vertex needs comes from its parent and from j's
// parent, so a single preorder pass does it.
// original: corpus/submissions/5127828.cpp

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
    if s[v.to_idx] == '(':
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
