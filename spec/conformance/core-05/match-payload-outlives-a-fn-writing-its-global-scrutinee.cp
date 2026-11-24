// expect: ok, output "ab\nnone"
// rule: 05.3.2
// A payload a match binds keeps its value when a fn called in the arm reassigns the global scrutinee (adversarial round 47).
enum Q = Ask(s: Str) | Skip
var q = Q.Ask("ab")
fn clear():
    q = Q.Skip
match q:
    Ask(s) =>
        clear()
        print(s)
    Skip => print("none")
match q:
    Ask(s) => print(s)
    Skip => print("none")
