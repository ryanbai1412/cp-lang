// Source: IOI 2020 P5 (Counting Mushrooms), https://dmoj.ca/problem/ioi20p5
// Problem: interactive; n <= 2e4 mushrooms of species A or B, mushroom 0 is A. use_machine(x)
//   on distinct labels returns the number of adjacent pairs of different species; at most 2e4
//   calls and total length 1e5. Return the number of A mushrooms.
// Solution: until 199 mushrooms are classified, take up to 5 unknowns at a time and place them
//   among known mushrooms in an arrangement found by brute force over permutations, so that two
//   queries separate all 2^k species masks (a two-level decision tree, cached per shape).
//   Then interleave the larger known same-species pool with unknowns: k unknowns per query,
//   ceil(res / 2) of them differ from the pool and the parity of res gives the last one.
//   A few hundred queries and O(n) total length.
// stdin here: n, then the hidden species s[0..n) (0 = A, 1 = B; s[0] == 0); use_machine is
//   simulated over them. stdout: the number of A mushrooms.
// original: corpus/submissions/4736004.cpp (grader function count_mushrooms)
// Bounds: n: 2..=20_000; hidden: 0..=1
let n: 2..=20_000 = read()
domain Mushroom = 0..n
let hidden: [Mushroom]0..=1 = read()
assert hidden[Mushroom(0)] == 0

fn use_machine(xs: List<Mushroom>) -> Int = xs.adjacent().count(p => hidden[p.0] != hidden[p.1])

// a query slot: a mushroom of known species, or the i-th of the unknowns being classified
enum Slot: Known(0..=1) | Unknown(Int)

fn species(s: Slot, mask: Int) -> Int:
    match s:
        Known(t) => t
        Unknown(i) => (mask >> i) & 1

fn answer(q: List<Slot>, mask: Int) -> Int = (1..q.len).count(i => species(q[i - 1], mask) != species(q[i], mask))

// the mask behind each of q's answers, if no two masks collide
fn split(masks: List<Int>, q: List<Slot>) -> ?List<?Int>:
    var mask_of: List<?Int> = [None for _ in 0..q.len]
    for m in masks:
        let r = answer(q, m)
        if mask_of[r] != None: return None
        mask_of[r] = m
    mask_of

// two queries tell the 2^k species masks of k unknowns apart, placed among x A's and y B's:
// the root's answer picks a leaf, whose own query (none when one mask is left) names the mask.
// Both are the first separating arrangement in permutation order.
struct Leaf(masks: List<Int>, q: List<Slot>, mask_of: List<?Int>)
struct Tree(root: List<Slot>, leaves: Map<Int, Leaf>)

fn first_arrangement(k: Int, x: Int, y: Int) -> List<Slot> =
    [Unknown(i) for i in 0..k] + [Known(0) for _ in 0..x] + [Known(1) for _ in 0..y]

fn decision_tree(k: Int, x: Int, y: Int) -> Tree:
    var root = first_arrangement(k, x, y)
    loop:
        let buckets = (0..1 << k).group(by: m => answer(root, m))
        var leaves: Map<Int, Leaf> = Map()
        for (r, masks) in buckets:
            var q = first_arrangement(k, x, y)
            loop:
                if let mask_of = split(masks, q):
                    leaves[r] = Leaf(masks, q, mask_of)
                    break
                if not q.next_permutation(): break
        if leaves.len == buckets.len: return Tree(root, leaves)
        assert root.next_permutation()

var known: [0..=1]List<Mushroom> = []
known[0].push(Mushroom(0))
var next = 1
var trees: Map<(Int, Int, Int), Tree> = Map()

fn ask(q: List<Slot>, fresh: List<Mushroom>) -> Int:
    var used: [0..=1]Int = 0
    var xs: List<Mushroom> = []
    for s in q:
        match s:
            Known(t) =>
                xs.push(known[t][used[t]])
                used[t] += 1
            Unknown(i) => xs.push(fresh[i])
    use_machine(xs)

// classify up to `want` unknowns with two queries, among x A's and y B's
fn classify(want: Int, x: Int, y: Int):
    let k = min(want, n - next)
    let fresh = [Mushroom(next + i) for i in 0..k]
    next += k
    if (k, x, y) not in trees: trees[(k, x, y)] = decision_tree(k, x, y)
    let tree = trees[(k, x, y)]
    let leaf = tree.leaves[ask(tree.root, fresh)]
    let mask = if leaf.masks.len == 1: leaf.masks[0] else: leaf.mask_of[ask(leaf.q, fresh)]!
    for (i, f) in fresh.items: known[(mask >> i) & 1].push(f)

// until 199 are known, classify a handful at a time
while next < n and next < 199:
    let (a, b) = (known[0].len, known[1].len)
    if a == 1 and b == 0: classify(1, 1, 0)
    elif a + b == 2: classify(2, a, b)
    elif a > 0 and b > 0: classify(5, min(a, 3), 4 - min(a, 3))
    else: classify(4, 4, 0)

// then interleave the larger known pool with as many unknowns: each but the last adds 0 or 2
// species changes as it matches the pool or not, the last 0 or 1
var a_count = known[0].len
while next < n:
    let side = if known[1].len > known[0].len: 1 else: 0
    let take = min(known[side].len, n - next)
    let fresh = [Mushroom(next + i) for i in 0..take]
    var xs: List<Mushroom> = []
    for i in 0..take:
        xs.push(known[side][i])
        xs.push(fresh[i])
    let res = use_machine(xs)
    let other = (res + 1) / 2
    a_count += if side == 0: take - other else: other
    known[if res % 2 == 1: 1 - side else: side].push(fresh.last!)
    next += take
print(a_count)
