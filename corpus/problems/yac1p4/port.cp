// Source: Yet Another Contest 1 P4 (No More Searching), https://dmoj.ca/problem/yac1p4
// limits: 1000 ms, 128 MB
// Problem: a string of n <= 250 letters A..Z (worth 1..26) is split into subsequences; one with
//   first value u and last value v scores u^2 - v^2. Print how many total scores are possible,
//   then each of them in ascending order.
// Solution: only first and last letters count, so a total is the sum of w^2 over left ends minus
//   over right ends, paired so each left end comes before its right end. Scan right to left
//   keeping, per count of unmatched right ends, the Bitset of reachable totals, offset by the
//   worth of the letters seen so it only shifts up. O(n^2 W / 64), W = 2 * total worth.
// original: corpus/submissions/4376742.cpp

let n: 1..=250 = read()
let word: Str = read()
let worth = [(c - 'A' + 1) * (c - 'A' + 1) for c in word]
let whole = worth.sum()

// Scan from the right. reach[open] holds the totals that leave `open`
// right ends still waiting for a left end. A set can only be shifted up,
// so a total is stored with the worth of every letter seen so far added
// to it: leaving a letter out shifts by its worth, adding it by twice
// that, and subtracting it by nothing.
domain Open = 0..=n
domain Lifted = 0..=2 * whole
var reach = [Open] _ => Bitset<Lifted>.new()
reach[Open.first!].add(Lifted.first!)
for w in worth.rev:
    var next = [Open] _ => Bitset<Lifted>.new()
    for open in Open:
        next[open].union(reach[open].shifted(w))
        if let more = Open.at(open + 1): next[open].union(reach[more].shifted(2 * w))
        if let less = Open.at(open - 1): next[open].union(reach[less])
    reach = next

let totals = reach[Open.first!]
print(totals.len)
for t in totals.each(): print(t.ord - whole)
