// Codeforces 2261 D2: give every restriction a random 64-bit key and xor it
// over its range, so two positions hash alike exactly when the same
// restrictions cover both. Equal neighbours in position order force a cut
// between them; the cuts are then paired up into intervals.
use std.seq.{scan}
@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
var diff = List.filled(n + 1, 0)
for _ in 0..m:
    let (l, r): (1..=n, 1..=n) = read()
    let key = rand(0, (1 << 62) - 1)
    diff[l - 1] ^= key
    diff[r] ^= key
let hash = scan(diff[0..n], (a, b) => a ^ b)

// next[i]: the nearest later position with the same hash as i, else n
var next = List.filled(n, n)
var (first_bad, last_bad): (?Int, ?Int) = (none, none)
let order = sorted(List(0..n), by: i => (hash[i], i))
for (a, b) in order.adjacent if hash[a] == hash[b]:
    next[a] min= b
    first_bad min= a
    last_bad max= b

var cuts = List<Int>()
if let f = first_bad and hash[f] == hash[last_bad!]: cuts.push(0)
for i in 0..n - 1:
    if next[i] == i + 1: cuts.push(i + 1)
    else: next[i + 1] min= next[i]
if cuts.len % 2 == 1: cuts.push(n)
let half = cuts.len / 2
print(half)
for i in 0..half: print(cuts[i] + 1, cuts[i + half])
