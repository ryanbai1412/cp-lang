// NOI 2020 P5, surreal trees: can a set of binary trees grow into all but finitely many trees?
// original: corpus/submissions/3803796.cpp
@multitest
// a trie over the spine of each tree; a spine step is one of four shapes
enum Step = OnlyLeft | OnlyRight | BothGoLeft | BothGoRight
struct Node:
    ends: Bool = false                  // some tree stops here
    next: [Step]?Int = none
var trie: List<Node> = [Node()]

fn step(o: Int, s: Step) -> Int:
    trie[o].next[s] ??= trie.push_id(Node())

let q: 1..=2e6 = read()
for _ in 0..q:
    let n: 1..=2e6 = read()
    domain V = 1..=n
    let kid: [V](l: ?V, r: ?V) = read(none: "0")
    fn leaf(v: V) -> Bool: kid[v] == (none, none)

    // a tree with a node whose two children both branch can never be the
    // only cover of anything, so its walk just stops there
    var at: ?V = V.first
    var o = 0
    while let v = at:
        at = none
        match kid[v]:
            (none, none) => trie[o].ends = true
            (l, none) => (at, o) = (l, step(o, OnlyLeft))
            (none, r) => (at, o) = (r, step(o, OnlyRight))
            (l, r) =>
                // a leaf sibling lets the walk go on down the other side
                if leaf(l!) and leaf(r!): trie[step(o, BothGoRight)].ends = true
                if leaf(r!): (at, o) = (l, step(o, BothGoLeft))
                elif leaf(l!): (at, o) = (r, step(o, BothGoRight))

// almost complete: every trie node either ends a tree or has all four shapes below it
fn full(o: ?Int) -> Bool:
    if let o = o: trie[o].ends or all(full(trie[o].next[s]) for s in Step) else: false
print(if full(0): "Almost Complete" else: "No")
