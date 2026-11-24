// Source: NOI 2017, Queueing Worms, https://dmoj.ca/problem/noi17p2
// limits: 2000 ms, 1024 MB
// Problem: n <= 2e5 worms of lengths 1..=6, each its own queue; m <= 5e5 operations: join the
//   queue ending at i to the one starting at j, split after i (at most 1e3 splits), or for a
//   digit string s and k <= 50 print the product over the length-k substrings t of s of the
//   number of worms whose next k lengths read t, mod 998244353 (sum |s| <= 1e7).
// Solution: read every operation first and keep, per length, a map from the 61-bit rolling
//   hash of each window a query asks about to its count, all hashes sharing one base. A join
//   adds the runs crossing the new joint (a suffix of u's queue plus a prefix of v's, at most 50
//   long) that some query asks about, and a split removes them; each run is made once between
//   splits, so that is O(50 n + 50^2 c) lookups in maps of only the queried windows. A query
//   multiplies its counts.
// Bounds: n: 1..=200_000; q: 1..=500_000; size: 1..=6; ops.u, ops.v: 1..=n; ops.k: 1..=longest

use std.strings.{Hash61, RollingHash61}

type Z = Mod<998_244_353>
let longest = 50
let rh = RollingHash61(List.filled(longest, 0))

struct Block(hash: Hash61, len: Int)
let empty_block = Block(rh.get(0..0), 0)
let one = [Block(RollingHash61([d], base: rh.base).all, 1) for d in 0..=6]

fn concat(a: Block, b: Block) -> Block = Block(rh.combine(a.hash, b.hash, b.len), a.len + b.len)

let (n, q): (1..=200_000, 1..=500_000) = read()
domain Worm = 1..=n
let size: [Worm]1..=6 = read()
enum Op:
    @tag(1) Join(u: Worm, v: Worm)
    @tag(2) Split(u: Worm)
    @tag(3) Ask(s: Str, k: 1..=longest)
let ops = read<[q]Op>()
fn windows(s: Str, k: Int) -> List<Hash61>:
    let hash = RollingHash61([c - '0' for c in s], base: rh.base)
    [hash.get(i..i + k) for i in 0..=s.len - k]

// Only runs some query asks about are counted, so the map holds just their windows.
var blocks: [1..=longest]HashMap<Hash61, Int> = []
var wanted: [1..=longest]Int = 0
var asked: List<List<Hash61>> = []
for op in ops:
    if let Ask(s, k) = op:
        wanted[k] += s.len - k + 1
        asked.push(windows(s, k))
for k in 1..=longest: blocks[k].reserve(wanted[k])
var at_ask = 0
for op in ops:
    if let Ask(_, k) = op:
        for h in asked[at_ask]: blocks[k][h] = 0
        at_ask += 1
at_ask = 0

fn bump(b: Block, by: Int):
    if blocks[b.len].contains(b.hash): blocks[b.len][b.hash] += by

var before: [Worm]?Worm = None
var after: [Worm]?Worm = None

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

fn joint(u: Worm, v: Worm, by: Int):
    let (left, right) = (reach(u, before, false), reach(v, after, true))
    for a in left:
        for b in right:
            if a.len + b.len > longest: break
            bump(concat(a, b), by)

for w in Worm: bump(one[size[w]], 1)
for op in ops:
    match op:
        Join(u, v) =>
            joint(u, v, 1)
            (after[u], before[v]) = (Some(v), Some(u))
        Split(u) =>
            let v = after[u]!
            joint(u, v, -1)
            (after[u], before[v]) = (None, None)
        Ask(_, k) =>
            print(asked[at_ask].map(h => Z(blocks[k][h])).prod())
            at_ask += 1
