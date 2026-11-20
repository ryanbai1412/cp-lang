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
domain Letter = 'a'..='z'

struct Node:
    kid: [Letter]?Int
    word: Bool
    last: Bool      // on the way to the word that is printed last

var trie = [Node([Letter] c => none, false, false)]

let n: Int = read()
domain Word = 1..=n
let words: [Word]Str = read()
var longest = ""
for w in Word:
    let word = words[w]
    if word.len > longest.len: longest = word
    var at = 0
    for c in word:
        if let next = trie[at].kid[Letter(c)]: at = next
        else:
            trie.push(Node([Letter] c => none, false, false))
            trie[at].kid[Letter(c)] = some(trie.len - 1)
            at = trie.len - 1
    trie[at].word = true

var at = 0
for c in longest:
    at = trie[at].kid[Letter(c)]!
    trie[at].last = true

var ops = ""
fn type_out(at: Int):
    if trie[at].word: ops.push('P')
    var kept: ?(Char, Int) = none
    for c in Letter:
        if let next = trie[at].kid[c]:
            if trie[next].last: kept = some((c.label, next))
            else:
                ops.push(c.label)
                type_out(next)
                ops.push('-')
    if let (c, next) = kept:
        ops.push(c)
        type_out(next)

type_out(0)
print(ops.len)
for op in ops: print(op)
