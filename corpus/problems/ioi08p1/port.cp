// Source: IOI 2008 P1 (Type Printer), https://dmoj.ca/problem/ioi08p1
// Problem: n <= 25000 distinct lowercase words of length <= 20. A printer adds a letter, removes
//   the last one ('-') or prints its word ('P'), and may end non-empty. Print the fewest
//   operations that print every word, in any order, then the operations, one per line.
// Solution: put the words in a trie and walk it, typing down and deleting up, printing at word
//   ends. Every edge costs 2 except those on the path to the last word, so end on a longest
//   word: at each node visit the child on that word's path last and never delete it.
//   O(total letters * 26).
// original: corpus/submissions/5034409.cpp
// Bounds: n: 1..=25_000
use std.strings.{Trie}

domain Letter = 'a'..='z'

var trie = Trie<Letter>.new()

let n: 1..=25_000 = read()
domain Word = 1..=n
let words: [Word]Str = read()
var longest = ""
for w in Word:
    let word = words[w]
    trie.add([Letter(c) for c in word])
    if word.len > longest.len: longest = word.to_str()

var on_longest: Set<Int> = Set()
var at = trie.root
for c in longest:
    at = trie.step(at, Letter(c))!
    on_longest.add(at)

var ops = ""
fn type_out(at: Int):
    if trie.ends_at(at) > 0: ops.push('P')
    var kept: ?(Char, Int) = None
    for c in Letter:
        if let next = trie.step(at, c):
            let ch = Char(c.label)
            if next in on_longest: kept = Some((ch, next))
            else:
                ops.push(ch)
                type_out(next)
                ops.push('-')
    if let (c, next) = kept:
        ops.push(c)
        type_out(next)

type_out(trie.root)
print(ops.len)
for op in ops: print(op)
