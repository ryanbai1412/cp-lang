// Source: SAC '22 Code Challenge 3 P5 (Chat Corrections), https://dmoj.ca/problem/sac22cc3p5
// limits: 500 ms, 1024 MB
// Problem: n <= 5e4 distinct banned words (<= 25 letters) and m <= 1000 messages (<= 500
//   letters), case-insensitive. For each message print how many distinct banned words occur in it.
// Solution: lowercase the words and drop repeats, build an Aho-Corasick automaton over them, and
//   count the distinct words among each message's matches. O(total length + matches), at most
//   25 matches per message position.
// original: corpus/submissions/4331687.cpp
use std.strings.{AhoCorasick}

domain Letter = 'a'..='z'
fn letters(s: Str) -> List<Letter> = [Letter(c.lower) for c in s]

let (n, m) = read<(1..=5e4, 1..=1000)>()
let words = read<[n]Str>().distinct(by: w => w.to_lower)
let ac = AhoCorasick<Letter>.new([letters(w) for w in words])
for _ in 0..m:
    print(ac.find_all(letters(read<Str>())).distinct(by: hit => hit.pattern).len)
