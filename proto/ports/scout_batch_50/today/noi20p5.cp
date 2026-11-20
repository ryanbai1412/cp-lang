// NOI 2020 P5, surreal trees: can a set of binary trees grow into all but finitely many trees? (buildable today-form)
// original: corpus/submissions/3803796.cpp
// The trie is two flat lists and the four shapes are Ints 0..4: the
// prototype has no List<struct> arena with push_id and no enum-indexed array.
let t: 1..=100 = read()
for _ in 0..t:
    // a trie over the spine of each tree; a spine step is one of four shapes:
    // 0 only a left child, 1 only a right child, 2 both and the right one is
    // a leaf (go left), 3 both and the left one is a leaf (go right)
    var kid: List<Int> = [0, 0, 0, 0]
    var ends: List<Bool> = [false]

    fn step(o: Int, shape: Int) -> Int:
        if kid[4 * o + shape] == 0:
            kid[4 * o + shape] = ends.len
            ends.push(false)
            for _ in 0..4: kid.push(0)
        kid[4 * o + shape]

    let q: 1..=2e6 = read()
    for _ in 0..q:
        let n: 1..=2e6 = read()
        var lc: [1..=n]Int = 0
        var rc: [1..=n]Int = 0
        for i in 1..=n:
            let l: 0..=n = read()
            let r: 0..=n = read()
            lc[i] = l
            rc[i] = r

        fn leaf(i: Int) -> Bool: lc[i] == 0 and rc[i] == 0

        // a tree with a node whose two children both branch can never be the
        // only cover of anything, so its walk just stops there
        var (i, o) = (1, 0)
        while i != 0:
            let (l, r) = (lc[i], rc[i])
            if l == 0 and r == 0:
                ends[o] = true
                i = 0
            elif l != 0 and r != 0 and leaf(l) and leaf(r):
                let (go_left, go_right) = (step(o, 2), step(o, 3))
                ends[go_left] = true
                ends[go_right] = true
                i = 0
            elif l != 0 and r != 0 and not leaf(l) and not leaf(r): i = 0
            else:
                // exactly one way down
                let shape = if r == 0: 0 elif l == 0: 1 elif leaf(r): 2 else: 3
                o = step(o, shape)
                i = if shape == 0 or shape == 2: l else: r

    // almost complete: every trie node either ends a tree or has all four shapes below it
    var ok = true
    var todo: List<Int> = [0]
    while let o = todo.pop():
        if ends[o]: continue
        for shape in 0..4:
            if kid[4 * o + shape] == 0: ok = false
            else: todo.push(kid[4 * o + shape])
    print(if ok: "Almost Complete" else: "No")
