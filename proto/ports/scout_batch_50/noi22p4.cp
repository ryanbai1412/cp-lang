// NOI 2022 P4, can rooted tree G become rooted tree H by deleting at most k <= 5 leaves, one at a time?
// original: corpus/submissions/4802121.cpp
let (tc, tests, k): (Int, 1..=500, 0..=5) = read()
@multitest(count: tests)
fn solve():
    // both trees share one node space, H after G
    var kids: List<List<Int>> = []
    var roots: List<Int> = []
    for _ in 0..2:
        let n: 1..=1e5 = read()
        let base = kids.len
        for _ in 0..n: kids.push([])
        for i in 0..n:
            let p: ?(1..=n) = read(none: "-1")
            if let p = p: kids[base + p - 1].push(base + i) else: roots.push(base + i)

    // shape[u]: one number per unordered subtree shape; size[u]: its nodes
    var names = Map<List<Int>, Int>()
    var shape = List.filled(kids.len, 0)
    var size = List.filled(kids.len, 1)
    fn name(u: Int):
        for v in kids[u]:
            name(v)
            size[u] += size[v]
        shape[u] = (names[sorted(shape[v] for v in kids[u])] ??= names.len)
    for r in roots: name(r)

    // prune(u, v): can G's subtree u lose leaves until it is H's subtree v?
    @memo
    fn prune(u: Int, v: Int) -> Bool:
        if shape[u] == shape[v]: return true
        if size[u] <= size[v] or size[u] - size[v] > k or kids[u].len < kids[v].len: return false
        // children of equal shape pair off; what is left on each side
        var theirs = SortedBag<(Int, Int)>((shape[c], c) for c in kids[v])
        var left: List<Int> = []
        for c in kids[u]:
            if let (s, d) = theirs.first_ge((shape[c], 0)) and s == shape[c]: theirs.remove((s, d))
            else: left.push(c)
        let right = [d for (s, d) in theirs]
        // every leftover child of u costs a deletion: it shrinks into its own
        // leftover child of v or disappears whole
        var taken = Subset<0..5>.empty
        fn place(at: Int) -> Bool:
            if at == right.len: return true
            for c in 0..left.len if c not in taken and prune(left[c], right[at]):
                taken = taken.with(c)
                defer taken = taken.without(c)
                if place(at + 1): return true
            false
        left.len <= size[u] - size[v] and place(0)

    print(if prune(roots[0], roots[1]): "Yes" else: "No")
