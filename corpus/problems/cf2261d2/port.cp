// Codeforces 2261 D2 — XOR-Hashed Restriction Cuts
//
// Give every restriction `[l, r]` a random 62-bit key and XOR it over its
// range via `diff`, so two positions share `hash` iff the exact same
// restrictions cover both.
//
// Every pair `(a, b)` of equal-hash positions must be separated by at least
// one cut. Sweeping `next[i]` (`?Int`, nearest later equal-hash partner,
// propagated forward when no cut is placed at `i + 1`) greedily places cuts
// at minimal positions.
//
// Pairing construction: we output `half = cuts.len / 2` intervals
// `[cuts[i] + 1, cuts[i + half]]`. Because every cut `c` is either a left
// boundary (`c + 1`) or a right boundary (`c`) of one of the `half` intervals,
// any two positions separated by at least one cut lie on opposite sides of at
// least one interval boundary, except possibly the outermost pair
// `(first_bad, last_bad)` if every cut lies strictly between them and both sit
// outside all intervals — so we prepend `0` when `hash[first_bad] == hash[last_bad]`,
// and append `n` when `cuts.len` is odd to pair the cuts evenly.

@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
var diff: [0..=n]Int = 0
for _ in 0..m:
    let (l, r): (1..=n, 1..=n) = read()
    let key = rand(0, (1 << 62) - 1)
    diff[l - 1] ^= key
    diff[r] ^= key
let hash = scan(diff[0..n], ^)

var next: [0..n]?Int = none
var (first_bad, last_bad): (?Int, ?Int) = (none, none)
let order = (0..n).sorted(by: i => (hash[i], i))
for (a, b) in order.adjacent if hash[a] == hash[b]:
    next[a] min= b
    first_bad min= a
    last_bad max= b

var cuts: List<Int> = []
if let f = first_bad and let l = last_bad and hash[f] == hash[l]: cuts.push(0)
for i in 0..n - 1:
    if next[i] == i + 1: cuts.push(i + 1)
    else: next[i + 1] min= next[i]
if cuts.len % 2 == 1: cuts.push(n)

let half = cuts.len / 2
print(half)
for i in 0..half: print(cuts[i] + 1, cuts[i + half])
