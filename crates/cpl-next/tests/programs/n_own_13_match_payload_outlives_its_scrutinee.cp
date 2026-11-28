// expect: ok, output "cd\nab 1 skip"
// A payload a match expression binds keeps its value when the arm writes the scrutinee, directly or as a `mut` param (R05.3.2).
enum Q = Ask(s: Str) | Skip
var q = Q.Ask("ab")
let u = match q:
    Ask(s) =>
        q = Q.Skip
        s
    Skip => "none"
fn go(mut r: Q) -> Int:
    match r:
        Ask(s) =>
            r = Q.Skip
            print(s)
            1
        Skip => 0
var g = Q.Ask("cd")
print(u, go(mut g), match g: Skip => "skip", Ask(_) => "ask")
