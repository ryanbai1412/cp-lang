// IOI '20 P5 (Mushrooms): count type-A mushrooms in an interactive machine returning
// the number of adjacent mismatched pairs in a query sequence.
// Phase 1 precomputes depth-2 decision trees via next_permutation (cached with `??=`)
// to classify batches of up to 5 mushrooms; Phase 2 interleaves the larger known bucket
// against remaining unknowns to count ~k mushrooms per query.
// original: corpus/submissions/4736004.cpp

use std.seq.{next_permutation}

@grader fn use_machine(x: List<Int>) -> Int

const B = 100

struct LeafDT:
    query: List<Int>
    res: List<?Int>

struct RootDT:
    query: List<Int>
    children: List<LeafDT>

fn eval_mask(mask: Int, perm: List<Int>) -> Int:
    fn bit_of(v: Int) -> Int = if v >= 2: (mask >> (v - 2)) & 1 else: v
    count(bit_of(a) != bit_of(b) for (a, b) in perm.adjacent)

fn find_perm<T>(k: Int, x: Int, y: Int, f: List<Int> -> ?T) -> ?T:
    var perm = List.filled(x, 0) + List.filled(y, 1) + [2 + i for i in 0..k]
    while true:
        if let hit = f(perm): return hit
        if not next_permutation(mut perm): return none

fn compute_dt(k: Int, x: Int, y: Int) -> RootDT:
    let tot = k + x + y
    fn try_root(perm: List<Int>) -> ?RootDT:
        var buckets: List<List<Int>> = List.filled(tot, [])
        for mask in 0..(1 << k): buckets[eval_mask(mask, perm)].push(mask)
        var kids: List<LeafDT> = []
        for grp in buckets:
            fn try_leaf(p2: List<Int>) -> ?LeafDT:
                var tbl: List<?Int> = List.filled(tot, none)
                for m in grp:
                    let r = eval_mask(m, p2)
                    if tbl[r] != none: return none
                    tbl[r] = m
                LeafDT(p2.copy(), tbl)
            let leaf = find_perm(k, x, y, try_leaf) ?? return none
            kids.push(leaf)
        RootDT(perm.copy(), kids)
    find_perm(k, x, y, try_root)!

@grader
fn count_mushrooms(n: Int) -> Int:
    var known: [Bool]List<Int> = []
    known[false].push(0)
    var pos = 1
    var cache = Map<(Int, Int, Int), RootDT>()

    fn ask_pattern(pat: List<Int>, vars: List<Int>) -> Int:
        var ptr: [Bool]Int = 0
        var seq: List<Int> = []
        for a in pat:
            if a < 2:
                let b = a == 1
                seq.push(known[b][ptr[b]])
                ptr[b] += 1
            else:
                seq.push(vars[a - 2])
        use_machine(seq)

    fn classify(want: Int, x: Int, y: Int):
        let k = min(want, n - pos)
        cache[(k, x, y)] ??= compute_dt(k, x, y)
        let dt = cache[(k, x, y)]!.copy()
        let vars = [pos + i for i in 0..k]
        pos += k
        let leaf = dt.children[ask_pattern(dt.query, vars)]
        let poss = [m! for m in leaf.res if m != none]
        let mask = if poss.len == 1: poss[0] else: leaf.res[ask_pattern(leaf.query, vars)]!
        for i in 0..k: known[((mask >> i) & 1) == 1].push(vars[i])

    while pos < n and pos < 2 * B - 1:
        let (a, b) = (known[false].len, known[true].len)
        if a == 1 and b == 0: classify(1, 1, 0)
        elif a + b == 2: classify(2, a, b)
        elif a > 0 and b > 0:
            let ax = min(a, 3)
            classify(5, ax, 4 - ax)
        else:
            classify(4, 4, 0)

    var ans = known[false].len
    while pos < n:
        let maj = known[true].len > known[false].len
        var seq: List<Int> = []
        for j in 0..known[maj].len:
            if pos >= n: break
            seq.push(known[maj][j])
            seq.push(pos)
            pos += 1
        let res = use_machine(seq)
        let diffs = (res + 1) / 2
        ans += if not maj: seq.len / 2 - diffs else: diffs
        known[if res % 2 == 1: not maj else: maj].push(seq.last!)
    ans
