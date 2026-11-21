// Codeforces 2261 G: sum over subarrays of (xor of the subarray) AND (and of
// the subarray). For a fixed right end the suffix ANDs take at most 25 values;
// each keeps a binary trie of the prefix xors of its left ends, and tries are
// merged when two suffix ANDs become equal. A group with AND `mask` adds, for
// each set bit b of `mask`, 2^b times the number of its left ends whose prefix
// xor differs from the current one at b: walking the trie along
// `pref ^ mask` counts exactly those as it passes each bit of `mask`.
use std.strings.{BitTrie}

const L = 25
type Word = 0..1 << L
struct Group(mask: Word, ends: BitTrie<L>)

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]Word = read()
let pref = a.scan(xor, init: 0)

var groups = List<Group>()
print(sum(
    for (r, x) in a.items:
        groups.push(Group(Word.last, BitTrie<L>.of(pref[r])))
        groups = groups.map(g => Group(g.mask & x, g.ends)).merge_runs(by: g => g.mask, (a, b) => Group(a.mask, a.ends.absorb(b.ends)))
        sum(g.ends.walk(pref[r + 1] ^ g.mask).sum((b, below) => if g.mask.bit(b): below.count << b else: 0) for g in groups)
))
