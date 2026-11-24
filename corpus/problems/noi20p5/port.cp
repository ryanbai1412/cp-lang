// Source: NOI 2020 P5 (Surreal), https://dmoj.ca/problem/noi20p5
// limits: 1000 ms, 512 MB
// Problem: T <= 100 instances, each a set of m binary trees (left and right children differ),
//   sum n, sum m and max height <= 2e6. grow(S) is every tree obtained from one in S by
//   replacing leaves with trees, repeatedly. Print "Almost Complete" if only finitely many trees
//   are outside grow(S), else "No".
// Solution: only trees where each node has at most one non-leaf child matter: one is a word over
//   four steps (left only, right only, both with the right a leaf, both with the left a leaf),
//   and a node with two leaf children ends both of its two-step words. Insert the words in a
//   std `Trie<Shape>`, dropping a tree at a node whose two children both branch. The answer is Almost
//   Complete iff every trie node reached from the root ends a word or has all four children:
//   O(sum n).
// original: corpus/submissions/3803796.cpp
// Bounds: m: 1..=2e6; n: 1..=2e6; kids.l, kids.r: 0..=n
use std.strings.{Trie}

// a spine step: only a left child, only a right child, or both with the other one a leaf
enum Shape: Left | Right | LeftOfLeaf | RightOfLeaf

@multitest
let m: 1..=2e6 = read()
var spines = Trie<Shape>.new()
repeat m:
    let n: 1..=2e6 = read()
    domain Node = 1..=n
    let kids: [Node](l: 0..=n, r: 0..=n) = read()

    fn kid(c: 0..=n) -> ?Node = if c == 0: None else: Node(c)
    fn leaf(i: Node) -> Bool = kids[i].l == 0 and kids[i].r == 0

    // a tree with a node whose two children both branch never covers anything alone, so it
    // adds no word
    var word: List<Shape> = []
    var at: ?Node = Node(1)
    while let i = at:
        at = None
        let (l, r) = (kid(kids[i].l), kid(kids[i].r))
        if l == None and r == None:
            spines.add(word)
        elif l == None:
            word.push(Right)
            at = r
        elif r == None:
            word.push(Left)
            at = l
        elif leaf(l!) and leaf(r!):
            spines.add(word + [LeftOfLeaf])
            spines.add(word + [RightOfLeaf])
        elif leaf(r!):
            word.push(LeftOfLeaf)
            at = l
        elif leaf(l!):
            word.push(RightOfLeaf)
            at = r

// almost complete: every trie node either ends a tree or has all four shapes below it
var ok = true
var todo = [spines.root]
while let o = todo.pop():
    if spines.ends_at(o) > 0: continue
    for s in Shape:
        if let c = spines.step(o, s):
            todo.push(c)
        else:
            ok = false
print(if ok: "Almost Complete" else: "No")
