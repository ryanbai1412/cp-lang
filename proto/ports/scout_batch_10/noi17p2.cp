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
use scout.hashing.{Hash, StrHash}
use std.mod.{Mod}

type Z = Mod<998244353>
let longest = 50

let (n, q): (Int, Int) = read()
domain Worm = 1..=n
let size: [Worm]Int = read()
var ahead: [Worm]?Worm = none
var behind: [Worm]?Worm = none
var blocks = Map<Hash, Int>()

fn count(block: Hash, by: Int):
    blocks[block] = (blocks.get(block) ?? 0) + by

/// The blocks of fewer than `longest` worms that start at `from` and go back along the queue
/// (`toward` = behind), or end at `from` and go forward (`toward` = ahead). Shortest first.
fn reach(from: Worm, toward: [Worm]?Worm, back: Bool) -> List<Hash>:
    var out: List<Hash> = []
    var (at, block) = (some(from), Hash.empty)
    while out.len < longest - 1:
        if let w = at:
            block = if back: block + Hash.one(size[w]) else: Hash.one(size[w]) + block
            out.push(block)
            at = toward[w]
        else: break
    out

/// Count, or with `by = -1` forget, every block that has v right behind u.
fn joint(u: Worm, v: Worm, by: Int):
    let (left, right) = (reach(u, ahead, false), reach(v, behind, true))
    for l in left:
        for r in right:
            if l.len + r.len > longest: break
            count(l + r, by)

for w in Worm: count(Hash.one(size[w]), 1)
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
        let hash = StrHash.of_items([c - '0' for c in s])
        var ways = Z(1)
        for i in 0..=s.len - k: ways = ways * Z(blocks.get(hash.of(i..i + k)) ?? 0)
        print(ways)
