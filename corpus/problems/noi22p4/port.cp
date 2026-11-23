// NOI 2022 P4, can rooted tree G become rooted tree H by deleting at most k <= 5 leaves, one at a time? (buildable today-form)
// original: corpus/submissions/4802121.cpp
// Both trees share one node space (H is offset by |G|) held in List<List<Int>>,
// and shapes are named through a Map<List<Int>, Int>: the prototype has no
// per-test domain inside a loop and no Rooted.from_parents.
let tc: Int = read()
let tests: 1..=500 = read()
let k: 0..=5 = read()
for _ in 0..tests:
    var kids: List<List<Int>> = []
    var roots: List<Int> = []
    for _ in 0..2:
        let n: 1..=1e5 = read()
        let base = kids.len
        for _ in 0..n: kids.push([])
        for i in 0..n:
            let p: -1..=n = read()
            if p == -1: roots.push(base + i)
            else: kids[base + p - 1].push(base + i)

    // shape[u]: one number per unordered subtree shape; size[u]: its nodes
    var names = Map<List<Int>, Int>()
    var shape: [0..kids.len]Int = 0
    var size: [0..kids.len]Int = 1
    fn name(u: Int):
        var below: List<Int> = []
        for v in kids[u]:
            name(v)
            size[u] += size[v]
            below.push(shape[v])
        below.sort()
        if names.get(below) == None: names[below] = names.len
        shape[u] = names[below]
    name(roots[0])
    name(roots[1])

    // prune(u, v): can G's subtree u lose leaves until it is H's subtree v?
    var memo = Map<(Int, Int), Bool>()
    fn prune(u: Int, v: Int) -> Bool:
        if shape[u] == shape[v]: return true
        if size[u] <= size[v] or size[u] - size[v] > k or kids[u].len < kids[v].len: return false
        if let known = memo.get((shape[u], shape[v])): return known
        // children of equal shape pair off; what is left on each side, by shape
        var mine = kids[u].sorted(by: c => shape[c])
        var theirs = kids[v].sorted(by: c => shape[c])
        var left: List<Int> = []
        var right: List<Int> = []
        var (i, j) = (0, 0)
        while i < mine.len or j < theirs.len:
            if j == theirs.len or (i < mine.len and shape[mine[i]] < shape[theirs[j]]):
                left.push(mine[i])
                i += 1
            elif i == mine.len or shape[theirs[j]] < shape[mine[i]]:
                right.push(theirs[j])
                j += 1
            else:
                i += 1
                j += 1
        // every leftover child of u costs a deletion: it shrinks into its own
        // leftover child of v or disappears whole
        var taken: List<Bool> = []
        for _ in left: taken.push(false)
        fn place(at: Int) -> Bool:
            if at == right.len: return true
            for c in 0..left.len:
                if not taken[c] and prune(left[c], right[at]):
                    taken[c] = true
                    if place(at + 1): return true
                    taken[c] = false
            false
        let ok = left.len <= size[u] - size[v] and place(0)
        memo[(shape[u], shape[v])] = ok
        ok

    print(if prune(roots[0], roots[1]): "Yes" else: "No")
