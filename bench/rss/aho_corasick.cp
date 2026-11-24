// n words of 8-10 letters over 3 of the 26 letters of their domain, then m texts walked by
// `step`: the automaton keeps one column per used letter, not a 26-cell row per state, and
// no per-state lists (sac22cc3p5).
use std.strings.{AhoCorasick}
domain Letter = 'a'..='z'
let (n, m): (1..=100_000, 1..=1000) = read()
let ac = AhoCorasick<Letter>.new([[Letter(c) for c in read<Str>()] for _ in 0..n])
for _ in 0..m:
    var s = ac.root
    var hits = 0
    for c in read<Str>():
        s = ac.step(s, Letter(c))
        if ac.hit(s): hits += 1
    print(hits)
