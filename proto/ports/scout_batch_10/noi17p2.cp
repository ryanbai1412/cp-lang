// NOI 2017 Queueing Worms: n worms, each with a length from 1 to 6, each a
// queue of one at first. "1 u v" puts the queue headed by v behind u, the
// last of its queue; "2 u" cuts the queue behind u; "3 s k" takes every
// block of k digits of s, counts the places where k worms standing in a row
// have those lengths, and prints the product of the counts modulo
// 998244353. k is at most 50 and there are few cuts.
//
// Keep a count for every block of at most 50 worms in a row. A join makes
// the blocks that cross it, a cut removes them: a hash of the worms up to u
// joined to a hash of the worms from v.
// original: corpus/submissions/5690636.cpp
use std.strings.{HashVal}

type Z = Mod<998244353>
let longest = 50
let rh = RollingHash.new(List.filled(longest, 0))
let one = [RollingHash.with_bases([d], rh.bases).all for d in 0..=6]

let (n, q): (Int, Int) = read()
domain Worm = 1..=n
let size: [Worm]Int = read()
var ahead: [Worm]?Worm = none
var behind: [Worm]?Worm = none
var blocks = Map<HashVal, Int>()

fn count(block: HashVal, by: Int):
    blocks[block] = (blocks.get(block) ?? 0) + by

/// The blocks of fewer than `longest` worms that start at `from` and go back along the queue
/// (`toward` = behind), or end at `from` and go forward (`toward` = ahead). Shortest first.
fn reach(from: Worm, toward: [Worm]?Worm, back: Bool) -> List<HashVal>:
    var out: List<HashVal> = []
    var (at, block) = (some(from), HashVal(0, 0))
    while out.len < longest - 1:
        if let w = at:
            let single = one[size[w]]
            block = if back: rh.combine(block, single, 1) else: rh.combine(single, block, out.len)
            out.push(block)
            at = toward[w]
        else: break
    out

/// Count, or with `by = -1` forget, every block that has v right behind u.
fn joint(u: Worm, v: Worm, by: Int):
    let (left, right) = (reach(u, ahead, false), reach(v, behind, true))
    for i in 0..left.len:
        for j in 0..right.len:
            if i + j + 2 > longest: break
            count(rh.combine(left[i], right[j], j + 1), by)

for w in Worm: count(one[size[w]], 1)
for _ in 0..q:
    let op: Int = read()
    if op == 1:
        let (u, v): (Worm, Worm) = read()
        joint(u, v, 1)
        (behind[u], ahead[v]) = (some(v), some(u))
    elif op == 2:
        let u: Worm = read()
        let v = behind[u]!
        joint(u, v, -1)
        (behind[u], ahead[v]) = (none, none)
    else:
        let (s, k): (Str, Int) = read()
        let hash = RollingHash.with_bases([c - '0' for c in s], rh.bases)
        var ways = Z(1)
        for i in 0..=s.len - k: ways = ways * Z(blocks.get(hash.slice(i, i + k)) ?? 0)
        print(ways)
