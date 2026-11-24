// Source: Yet Another Contest 1 P4 (No More Searching), https://dmoj.ca/problem/yac1p4
// limits: 1000 ms, 128 MB
// Problem: a string of n <= 250 letters A..Z (worth 1..26) is split into subsequences; one with
//   first value u and last value v scores u^2 - v^2. Print how many total scores are possible,
//   then each of them in ascending order.
// Solution: only first and last letters count, so a total is the sum of w^2 over left ends minus
//   over right ends, paired so each left end comes before its right end. Scan right to left
//   keeping, per count of right ends still open, the Bitset of reachable totals. A Bitset only
//   shifts up, so each total is stored lifted by the worth of the letters seen: skipping a
//   letter shifts by w, adding it as a left end by 2w, as a right end by 0. O(n^2 W / 64),
//   W = 2 * total worth.
// original: corpus/submissions/4376742.cpp

let n = read<1..=250>()
let word = read<Str>()
let worth = [(c - 'A' + 1) * (c - 'A' + 1) for c in word]
let whole = worth.sum()

domain Open = 0..=n
domain Lifted = 0..=2 * whole
var reach = [Open] _ => Bitset<Lifted>.new()
reach[Open.first!].add(Lifted.first!)
// In place, opens ascending: `below` keeps the step's old `reach[open - 1]`.
for (i, w) in worth.items.rev:
    var below = Bitset<Lifted>.new()
    for open in Open.first!..=Open(min(i, n - i)):
        var b = if let more = Open.at(open + 1): reach[more].shifted(w) else: Bitset<Lifted>.new()
        b.union(reach[open])
        b.shift_up(w)
        b.union(below)
        swap(mut below, mut reach[open])
        reach[open] = b

let totals = reach[Open.first!]
print(totals.len)
for t in totals.each(): print(t.ord - whole)
