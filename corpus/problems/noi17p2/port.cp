// Source: NOI 2017, Queueing Worms, https://dmoj.ca/problem/noi17p2
// Problem: n <= 2e5 worms of lengths 1..=6, each its own queue; m <= 5e5 operations: join the
//   queue ending at i to the one starting at j, split after i (at most 1e3 splits), or for a
//   digit string s and k <= 50 print the product over the length-k substrings t of s of the
//   number of worms whose next k lengths read t, mod 998244353 (sum |s| <= 1e7).
// Solution: count every run of at most 50 consecutive worms in a Map keyed by rolling hash. A
//   join adds the runs crossing the new joint (a suffix of i's queue plus a prefix of j's, at
//   most 50 long) and a split removes them; each run is made once between splits, so that is
//   O(50 n + 50^2 c) map updates. A query hashes s and multiplies the counts of its windows.
// Bounds: n: 1..=200_000; q: 1..=500_000; size: 1..=6

use std.strings.{HashVal}

type Z = Mod<998_244_353>
let longest = 50
let rh = RollingHash.new(List.filled(longest, 0))

struct Block(hash: HashVal, len: Int)
let empty_block = Block(HashVal(0, 0), 0)
let one = [Block(RollingHash.new([d], bases: rh.bases).all, 1) for d in 0..=6]

fn concat(a: Block, b: Block) -> Block = Block(rh.combine(a.hash, b.hash, b.len), a.len + b.len)

let (n, q): (1..=200_000, 1..=500_000) = read()
domain Worm = 1..=n
let size: [Worm]1..=6 = read()
var before: [Worm]?Worm = None
var after: [Worm]?Worm = None
var blocks = Counter<HashVal>()

/// Blocks of fewer than `longest` worms starting at `from` along `toward`
/// (`back = true`: extending on the right; `back = false`: prepending on the left).
fn reach(from: Worm, toward: [Worm]?Worm, back: Bool) -> List<Block>:
    var out: List<Block> = []
    var (at, block) = (Some(from), empty_block)
    while out.len < longest - 1:
        if let w = at:
            block = if back: concat(block, one[size[w]]) else: concat(one[size[w]], block)
            out.push(block)
            at = toward[w]
        else: break
    out

/// Count (`by = 1`) or forget (`by = -1`) every block with `v` right behind `u`.
fn joint(u: Worm, v: Worm, by: Int):
    let (left, right) = (reach(u, before, false), reach(v, after, true))
    for a in left:
        for b in right:
            if a.len + b.len > longest: break
            blocks.add(concat(a, b).hash, by)

for w in Worm: blocks.add(one[size[w]].hash, 1)
repeat q:
    match read<1..=3>():
        1 =>
            let (u, v): (Worm, Worm) = read()
            joint(u, v, 1)
            (after[u], before[v]) = (Some(v), Some(u))
        2 =>
            let u: Worm = read()
            let v = after[u]!
            joint(u, v, -1)
            (after[u], before[v]) = (None, None)
        _ =>
            let (s, k): (Str, 1..=longest) = read()
            let hash = RollingHash.new([c - '0' for c in s], bases: rh.bases)
            print((0..=s.len - k).map(i => Z(blocks[hash.slice(i, i + k)])).prod())
