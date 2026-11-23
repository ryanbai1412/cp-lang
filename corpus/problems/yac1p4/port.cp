// Yet Another Contest 1 P4: letter i of a word is worth the square of its
// place in the alphabet. Pick disjoint pairs of positions i < k; a pair
// scores w - worth[k]. Print every total that can be reached.
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
        if let more = open + 1: next[open].union(reach[more].shifted(2 * w))
        if let less = open - 1: next[open].union(reach[less])
    reach = next

let totals = reach[Open.first!]
print(totals.len)
for t in totals.each(): print(t.to_idx - whole)
