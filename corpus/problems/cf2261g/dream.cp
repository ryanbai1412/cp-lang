// Codeforces 2261 G: sum over subarrays of (xor of the subarray) AND (and of
// the subarray). For a fixed right end the suffix ANDs take at most 25 values;
// each keeps a binary trie of the prefix xors of its left ends, and tries are
// merged when two suffix ANDs become equal. A group with AND `mask` adds, for
// each set bit b of `mask`, 2^b times the number of its left ends whose prefix
// xor differs from the current one at b: walking the trie along
// `pref ^ mask` counts exactly those as it passes each bit of `mask`.
use std.bitset.{BitTrie}

const L = 25
const FULL = (1 << L) - 1
type Word = 0..=FULL
struct Group(mask: Word, ends: BitTrie<L>)

fn shrink_groups(groups: List<Group>, x: Word) -> List<Group>:
    var out = List<Group>()
    for run in groups.runs(by: g => g.mask & x):
        var ends = BitTrie<L>.new()
        for g in run:
            var e = g.ends
            ends.absorb(mut e)
        out.push(Group(run.first!.mask & x, ends))
    out

fn group_contrib(g: Group, pref: Word) -> Int =
    sum(count << bit for (bit, count) in g.ends.walk(pref ^ g.mask) if g.mask.bit(bit))

@!multitest
let n: 1..=4e5 = read()
let a: [0..n]Word = read()
let pref = a.scan(^)

var groups = List<Group>()
var total = 0
for (r, x) in a.items:
    groups.push(Group(FULL, BitTrie<L>.of(pref.cut(r))))
    groups = shrink_groups(groups, x)
    total += sum(group_contrib(g, pref.cut(r + 1)) for g in groups)
print(total)
