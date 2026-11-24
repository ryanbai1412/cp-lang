use std.segtree.{MergeTree, MergeTrees}
let n: Int = read()
domain V = 0..n
var s = 0

for _ in 0..1 << 17:
    var e = MergeTrees<V, Int>() // timed: segtree.MergeTrees.new
    s += 1

let leaf: (Int, Int, Int, Int) -> Int = (x, y, px, py) => x + y // within: segtree.MergeTrees.merge
for _ in 0..8:
    var p = MergeTrees<V, Int>()
    var trees: List<MergeTree<V>> = []
    for i in 0..n:
        var tr = MergeTree<V>()
        p.set(mut tr, V(i * 7919 % n), i) // timed: segtree.MergeTrees.set
        trees.push(tr)
    var m = trees[0]
    for i in 1..n:
        m = p.merge(m, trees[i], leaf) // timed: segtree.MergeTrees.merge
    for i in 0..n:
        s += p.sum(m, V(i / 2)..V(i / 2 + n / 2)) // timed: segtree.MergeTrees.sum
        s += p.get(m, V(i * 13 % n)) // timed: segtree.MergeTrees.get
        s += p.total(m) // timed: segtree.MergeTrees.total
        p.scale(m, 1) // timed: segtree.MergeTrees.scale
print(s % 1000000007)
