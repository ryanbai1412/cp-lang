// Source: New Year's '18 P7 (Tree of Life), https://dmoj.ca/problem/year2018p7
// limits: 1000 ms, 256 MB
// Problem: a tree rooted at 1 with N <= 20000 vertices, each with a DNA word (total length S
//   <= 1e5). Two vertices are distant when neither is an ancestor of the other. Print the
//   longest common substring of the words of two distant vertices, or -1 if no pair exists.
// Solution: if a block of some length is shared, so is every shorter one: binary search the
//   length. For one length, collect the rolling hashes of the blocks in every subtree, pouring
//   the smaller child set into the larger; a hash already there when two branches meet is a
//   shared block (a vertex's own blocks join after its children, unchecked). Length 0 tests
//   whether a distant pair exists. O(S log S log S) expected.
// original: corpus/submissions/4899354.cpp
// Bounds: n: 1..=20000; parent: V
use std.strings.{HashVal}

let n: 1..=20000 = read()
domain V = 1..=n
var kids: [V]List<V> = [V] v => []
for v in 2..=n:
    let parent: V = read()
    kids[parent].push(V(v))
let word: [V]Str = read()
let bases = (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
let hash = [V] v => RollingHash.new(word[v], bases:)

/// Do two branches of some vertex share a block of `len` letters?
fn shared(len: Int) -> Bool:
    var found = false
    /// The blocks of `len` letters in the words at and below u.
    fn blocks(u: V) -> Set<HashVal>:
        var all = Set<HashVal>()
        for v in kids[u]:
            var branch = blocks(v)
            if branch.len > all.len: swap(mut all, mut branch)
            for block in branch:
                if all.contains(block): found = true
                all.add(block)
        for i in 0..=word[u].len - len: all.add(hash[u].slice(i, i + len))
        all
    blocks(V(1))
    found

// The longest length that is shared; -1 if not even the empty block is.
print((0..=[word[v].len for v in V].max()!).last_true(shared) ?? -1)
