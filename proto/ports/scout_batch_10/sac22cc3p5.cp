// SAC 2022 Code Challenge 3 P5: n words and m texts, letters in either
// case, which does not count. For each text print how many different words
// occur in it. A word has at most 25 letters and a text at most 500.
//
// Put the words in a trie. From every place in a text walk down the trie as
// far as the text allows: every word node on the way is a word that occurs.
// A node remembers the last text it was counted for. The original hashes
// every block of up to 25 letters into a hand-built table instead.
// original: corpus/submissions/4340958.cpp
domain Letter = 'a'..='z'

struct Node:
    kid: [Letter]?Int
    word: Bool
    counted: Int    // the last text this word was counted for

var trie = [Node([Letter] c => none, false, 0)]

fn letter(c: Char) -> Letter = if c.is_upper: Letter.nth(c - 'A') else: Letter(c)

let (n, m): (Int, Int) = read()
for _ in 0..n:
    let word: Str = read()
    var at = 0
    for c in word:
        if let next = trie[at].kid[letter(c)]: at = next
        else:
            trie.push(Node([Letter] c => none, false, 0))
            trie[at].kid[letter(c)] = some(trie.len - 1)
            at = trie.len - 1
    trie[at].word = true

for text in 1..=m:
    let s: Str = read()
    var found = 0
    for start in 0..s.len:
        var at = 0
        for i in start..s.len:
            if let next = trie[at].kid[letter(s[i])]: at = next
            else: break
            if trie[at].word and trie[at].counted != text:
                trie[at].counted = text
                found += 1
    print(found)
