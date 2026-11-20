// New Year's 2018 P7: a rooted tree with a word at every vertex. Find the
// longest block of letters that occurs in the words of two vertices neither
// of which is above the other; print its length, or -1 if there is no such
// pair of vertices.
//
// If a block of some length is shared, so is every shorter one: binary
// search the length. For one length, collect the hashes of the blocks in
// every subtree, pouring the smaller set into the larger; a hash that is
// already there when two branches meet is a shared block.
// original: corpus/submissions/4899354.cpp
use scout.hashing.{Hash, StrHash}

let n: Int = read()
domain V = 1..=n
var kids: [V]List<V> = [V] v => []
for v in 2..=n:
    let parent: V = read()
    kids[parent].push(V(v))
let word: [V]Str = read()
let hash = [V] v => StrHash.new(word[v])

/// Do two branches of some vertex share a block of `len` letters?
fn shared(len: Int) -> Bool:
    var found = false
    /// The blocks of `len` letters in the words at and below u.
    fn blocks(u: V) -> Set<Hash>:
        var all = Set<Hash>()
        for v in kids[u]:
            var branch = blocks(v)
            if branch.len > all.len: swap(mut all, mut branch)
            for block in branch:
                if all.contains(block): found = true
                all.add(block)
        for i in 0..=word[u].len - len: all.add(hash[u].of(i..i + len))
        all
    blocks(V(1))
    found

// The longest length that is shared; -1 if not even the empty block is.
var (lo, hi) = (-1, max(word[v].len for v in V))
while lo < hi:
    let mid = (lo + hi + 1) / 2
    if shared(mid): lo = mid
    else: hi = mid - 1
print(lo)
