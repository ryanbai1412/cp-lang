// IOI 2008 Type Printer: n different lowercase words (n <= 25000, at most 20
// letters each) are to be printed, in any order, on a printer that can add a
// letter to the end of its line, take the last letter away ('-'), or print
// the line ('P'). Letters may be left on the line at the end. Print the
// fewest operations and then the operations, one to a line.
//
// The words form a trie and the printer walks it, typing on the way down and
// deleting on the way up. Every trie edge is traversed twice (down + '-')
// except the edges on the path to the final word, which are never deleted, so
// minimizing operations leaves a longest word last: at every node, defer the
// child on the longest word's path until after all other branches.
// original: corpus/submissions/5034409.cpp
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
