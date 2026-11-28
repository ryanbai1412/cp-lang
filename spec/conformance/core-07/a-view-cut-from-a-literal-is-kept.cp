// expect: ok, output "4 2 b 6"
// rule: 07.5.10
// A literal is the program's value, not a call's temporary: a view cut from one, or from a
// view of one, is kept past its statement.
let t = "abcdef"[1..5].rebase
let u = [5, 6, 7][1..].rebase
print(t.len, u.len, t[t.keys.first!], u[u.keys.first!])
