// ioi18p1 Combo: find a hidden word over A B X Y whose first letter never comes back, in n + 2
// presses. Two presses find a letter (which pair, then which of the pair); one press settles each
// middle letter: with the other three letters a, b, c, the press "s a | s b a | s b b | s b c"
// grows by 1 for a, 2 for b and 0 for c. The first and last letters take two presses each.
//
// The task is a grader function and the judge supplies `press`. `cpl run` has no local stub
// for that, so this plays the sample grader: it reads S, answers each press the way the judge
// does, and prints `Accepted: q`, q the number of presses.
use std.strings.{prefix_matches}

let word = read<Str of "ABXY">()
assert 1 <= word.len <= 2000
var presses = 0
fn press(p: Str) -> Int:
    presses += 1
    assert presses <= 8000 and p.len <= 4 * word.len
    prefix_matches(p, word).max() ?? 0

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

assert guess_sequence(word.len) == word
print("Accepted:", presses)
