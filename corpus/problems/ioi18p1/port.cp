// Source: IOI 2018 P1 (Combo), https://dmoj.ca/problem/ioi18p1
// Problem: interactive. A hidden word S of N <= 2000 letters over A B X Y whose first letter
//   never appears again; press(p), |p| <= 4N, returns the longest prefix of S that is a substring
//   of p. Find S; full score needs at most N + 2 presses.
// Solution: a letter in two presses: press(s+A+s+B) says the pair {A, B} or {X, Y}, then press
//   one of the pair. That finds the first letter; a, b, c are the other three. Each middle letter
//   takes one press of s+a + s+b+a + s+b+b + s+b+c (length 4|s| + 6 <= 4N), which scores |s| + 1
//   for a, |s| + 2 for b, |s| for c. The last letter takes two presses again: N + 2 in all.
// original: corpus/submissions/4707803.cpp
// Bounds: n: 1..=2000
let n: 1..=2000 = read()
fn press(p: Str) -> Int = ask("press", p)

// One letter after s in two presses: is it in the pair A B, then which of the pair.
fn letter_after(s: Str) -> Char:
    let hit = (p: Str) => press(p) > s.len
    if hit("{s}A{s}B"): (if hit("{s}A"): 'A' else: 'B') else: (if hit("{s}X"): 'X' else: 'Y')

fn guess_sequence(n: Int) -> Str:
    let first = letter_after("")
    let o = "ABXY".filter(ch => ch != first).collect()
    let (a, b, c) = (o[0], o[1], o[2])
    var s = Str([first])
    for _ in 2..n:
        let grew = press("{s}{a}{s}{b}{a}{s}{b}{b}{s}{b}{c}") - s.len
        s.push([c, a, b][grew])
    if n >= 2: s.push(letter_after(s))
    s

print("guess", guess_sequence(n))
