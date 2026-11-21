// SAC 2022 Code Challenge 3 P5: n words and m texts, letters in either
// case, which does not count. For each text print how many different words
// occur in it. A word has at most 25 letters and a text at most 500.
//
// Deduplicate the lowercased dictionary and build an Aho-Corasick automaton;
// for each text, count distinct matched word IDs.
// original: corpus/submissions/4340958.cpp
use std.strings.{AhoCorasick}

domain Letter = 'a'..='z'

fn to_letters(s: Str) -> List<Letter> = [Letter(c.lower) for c in s]

let (n, m): (Int, Int) = read()
var seen_words = Set<Str>()
var words: List<List<Letter>> = []
for _ in 0..n:
    let raw: Str = read()
    var w = ""
    for c in raw: w.push(c.lower)
    if w not in seen_words:
        seen_words.insert(w)
        words.push(to_letters(w))
let ac = AhoCorasick<Letter>.new(words)

for _ in 0..m:
    var matched = Set<Int>()
    for (_, id) in ac.matches(to_letters(read())):
        matched.insert(id)
    print(matched.len)
