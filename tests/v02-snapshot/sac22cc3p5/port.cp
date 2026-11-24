// Source: SAC '22 Code Challenge 3 P5 (Chat Corrections), https://dmoj.ca/problem/sac22cc3p5
// Problem: n <= 5e4 distinct banned words (<= 25 letters) and m <= 1000 messages (<= 500
//   letters), case-insensitive. For each message print how many distinct banned words occur in it.
// Solution: lowercase and deduplicate the words, build an AhoCorasick automaton over them, and
//   collect the word IDs of each message's matches in a Set. O(total length + matches), at most
//   25 matches per message position.
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
        seen_words.insert(w.clone())
        words.push(to_letters(w))
let ac = AhoCorasick<Letter>.new(words)

for _ in 0..m:
    var matched = Set<Int>()
    for (_, id) in ac.find_all(to_letters(read())):
        matched.insert(id)
    print(matched.len)
