// expect: ok, output "abc 2\nzz 9 abc\n5 hello\nnow stop"
// rule: 05.3.2
// A payload a match binds keeps its value when the arm reassigns the scrutinee, directly or through a `mut` call, so codegen may lend it only when nothing in the arm can change the scrutinee.
// copies: 2
enum Ev = Ask(s: Str, k: Int) | Stop
var e = Ev.Ask("abc", 2)
var log: List<Str> = []
match e:
    Ask(s, k) =>
        e = Ev.Ask("zz", 9)
        print(s, k)
        log.push(s)
    Stop => print("stop")
match e:
    Ask(s, k) => print(s, k, log)
    Stop => print("stop")
fn reset(mut x: Ev):
    x = Ev.Stop
var f = Ev.Ask("hello", 1)
match f:
    Ask(s, _) =>
        reset(mut f)
        print(s.len, s)
    Stop => print("stop")
print(match f: Stop => "now stop", Ask(_, _) => "ask")
