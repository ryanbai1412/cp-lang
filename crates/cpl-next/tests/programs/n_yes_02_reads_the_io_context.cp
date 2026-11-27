// expect: ok, output "YES NO\nYes No Impossible\nYes NO\nPossible YES\nok NO"
// rule: R10.3.3 R09.1.1
// The prelude's `yes` is std.io's, so it reads `io.answers` from the enclosing `with io`: also
// through a caller that declares `uses io`, never through one that does not, and an `answers:`
// argument still wins.
fn follows(ok: Bool) -> Str uses io = yes(ok)
fn fixed(ok: Bool) -> Str = yes(ok)
print(yes(true), yes(false))
with io(answers: ("Yes", "No")):
    print(yes(true), follows(false), yes(false, answers: ("Possible", "Impossible")))
    print(follows(true), fixed(false))
print(yes(true, answers: ("Possible", "Impossible")), follows(true))
with io(answers: ("ok", "bad")):
    print(yes(true), yes(false, answers: ("YES", "NO")))
