// NOI 2017 Queueing Worms: `n` worms, each with a length from 1 to 6, each a
// queue of one at first. "1 u v" puts the queue headed by `v` behind `u`, the
// last of its queue; "2 u" cuts the queue behind `u`; "3 s k" takes every
// block of `k` digits of `s`, counts the places where `k` worms standing in a
// row have those lengths, and prints the product of the counts modulo
// 998244353. `k` is at most 50 and there are few cuts.
//
// Maintain a count for every contiguous block of at most 50 worms. A join makes
// the blocks that cross `(u, v)` and a cut removes them: each crossing block is
// a suffix ending at `u` concatenated with a prefix starting at `v` of total
// length at most 50. `Block(hash, len)` keeps every hash paired with its length
// so `concat` never passes a length by hand.

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
var ahead: [Worm]?Worm = None
var behind: [Worm]?Worm = None
var blocks = Map<HashVal, Int>()

fn count(block: Block, by: Int):
    blocks[block.hash] = (blocks.get(block.hash) ?? 0) + by

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
    let (left, right) = (reach(u, ahead, false), reach(v, behind, true))
    for a in left:
        for b in right:
            if a.len + b.len > longest: break
            count(concat(a, b), by)

for w in Worm: count(one[size[w]], 1)
repeat q:
    match read<Int>():
        1 =>
            let (u, v): (Worm, Worm) = read()
            joint(u, v, 1)
            (behind[u], ahead[v]) = (Some(v), Some(u))
        2 =>
            let u: Worm = read()
            let v = behind[u]!
            joint(u, v, -1)
            (behind[u], ahead[v]) = (None, None)
        _ =>
            let (s, k): (Str, Int) = read()
            let hash = RollingHash.new([c - '0' for c in s], bases: rh.bases)
            var ways = Z(1)
            for i in 0..=s.len - k:
                ways *= Z(blocks.get(hash.slice(i, i + k)) ?? 0)
            print(ways)
