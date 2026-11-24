// Source: IOI 2018 P1 (Combo), https://dmoj.ca/problem/ioi18p1
// Problem: interactive. A hidden word S of N <= 2000 letters over A B X Y whose first letter
//   never appears again; press(p), |p| <= 4N, returns the longest prefix of S that is a substring
//   of p. Find S; full score needs at most N + 2 presses.
// Solution: a letter in two presses: press(s+A+s+B) says the pair {A, B} or {X, Y}, then press
//   one of the pair. That finds the first letter; a, b, c are the other three. Each middle letter
//   takes one press of s+a + s+b+a + s+b+b + s+b+c (length 4|s| + 6 <= 4N), which scores |s| + 1
//   for a, |s| + 2 for b, |s| for c. The last letter takes two presses again: N + 2 in all.
// The task is a grader function and the judge supplies `press`; this plays the sample grader:
// it reads S, answers each press the way the judge does, and prints `Accepted: q`.
// original: corpus/submissions/4707803.cpp
use std.strings.{prefix_matches}

const BUTTONS = "ABXY"
let word = read<Str of "ABXY">()
assert 1 <= word.len <= 2000

var presses = 0
fn press(p: Str) -> Int:
    presses += 1
    assert presses <= 8000 and p.len <= 4 * word.len
    prefix_matches(p, word).max() ?? 0

// One letter in two presses: is it one of the first pair, then which one.
fn letter_after(s: Str) -> Char:
    let pair = if press("{s}A{s}B") > s.len: "AB" else: "XY"
    if press("{s}{pair[0]}") > s.len: pair[0] else: pair[1]

fn guess_sequence(n: Int) -> Str:
    let first = letter_after("")
    if n == 1: return Str([first])
    let o = BUTTONS.filter(x => x != first).collect()
    let (a, b, c) = (o[0], o[1], o[2])
    var s = Str([first])
    // One press per middle letter: s+a scores 1, s+b+anything scores 2, c scores 0.
    for _ in 1..n - 1:
        let got = press("{s}{a}{s}{b}{a}{s}{b}{b}{s}{b}{c}") - s.len
        s.push([c, a, b][got])
    s + Str([letter_after(s)])

assert guess_sequence(word.len) == word
print("Accepted:", presses)
