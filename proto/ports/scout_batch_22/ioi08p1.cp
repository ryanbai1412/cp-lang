// IOI 2008 Type Printer: n different lowercase words (n <= 25000, at most 20
// letters each) are to be printed, in any order, on a printer that can add a
// letter to the end of its line, take the last letter away ('-'), or print
// the line ('P'). Letters may be left on the line at the end. Print the
// fewest operations and then the operations, one to a line.
//
// The words form a trie and the printer walks it, typing on the way down and
// deleting on the way up. Only the last word is never deleted, so the walk
// ends on a longest word: at every node the branch towards that word goes
// last.
// original: corpus/submissions/5034409.cpp
use std.strings.{Trie}

domain Letter = 'a'..='z'

var trie = Trie<Letter>.new()

let n: Int = read()
domain Word = 1..=n
let words: [Word]Str = read()
var longest = ""
for w in Word:
    let word = words[w]
    if word.len > longest.len: longest = word
    let _ = trie.add([Letter(c) for c in word])

var on_longest = List.filled(trie.len, false)
var at = trie.root
for c in longest:
    at = trie.step(at, Letter(c))!
    on_longest[at] = true

var ops = ""
fn type_out(at: Int):
    if trie.ends_at(at) > 0: ops.push('P')
    var kept: ?(Char, Int) = none
    for c in Letter:
        if let next = trie.step(at, c):
            if on_longest[next]: kept = some((Char(c.label), next))
            else:
                ops.push(Char(c.label))
                type_out(next)
                ops.push('-')
    if let (c, next) = kept:
        ops.push(c)
        type_out(next)

type_out(trie.root)
print(ops.len)
for op in ops: print(op)
