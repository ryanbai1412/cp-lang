// IOI 2018 Combo: a hidden word of n letters over A B X Y, whose first
// letter never appears again. press(p) returns the longest prefix of the
// word that occurs in p (p at most 4n long). Find the word in n + 2 presses.
//
// IDEAL SKETCH, not run: the prototype has no `protocol` / `@interactor`.
// The solution below is complete; the judge half is what `cpl stress` needs.
// original: corpus/submissions/4707803.cpp
const BUTTONS = "ABXY"

protocol Combo:
    fn press(p: Str) -> Int
    limit press: n + 2
    require p.len <= 4 * n  // a call that breaks this fails the run

// The hidden word. Its rule ("the first letter never comes back") is not a
// type: the generator builds it that way, and `valid` states the rule once
// so that shrinking and hand-written cases are held to it too.
@interactor(Combo)
struct Judge(word: Str):
    fn gen(n: Int) -> Judge:
        let first = rand_of(BUTTONS)
        let rest = [c for c in BUTTONS if c != first]
        Judge(first + Str(rand_of(rest) for _ in 1..n))
    fn valid(self) -> Bool = all(c != word[0] for c in word[1..])

    fn press(mut self, p: Str) -> Int = [k for k in 0..=word.len if p.contains(word[..k])].max()!
    fn answer_ok(self, got: Str) -> Bool = got == word

// One letter in two presses: is it one of the first pair, then which one.
fn letter_after(s: Str, judge: Combo) -> Char:
    let pair = if judge.press(s + 'A' + s + 'B') > s.len: "AB" else: "XY"
    if judge.press(s + pair[0]) > s.len: pair[0] else: pair[1]

@graded fn guess_sequence(n: Int, judge: Combo) -> Str:
    let first = letter_after("", judge)
    if n == 1: return Str(first)
    let (a, b, c) = [x for x in BUTTONS if x != first]
    var s = Str(first)
    // One press per middle letter: s+a scores 1, s+b+anything scores 2, c scores 0.
    for _ in 1..n - 1:
        let got = judge.press(s + a + s + b + a + s + b + b + s + b + c) - s.len
        s.push([c, a, b][got])
    s + letter_after(s, judge)
