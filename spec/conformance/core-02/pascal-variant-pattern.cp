// expect: ok, output "2"
// rule: 02.7.1
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
let e = Ev.Ask(1, 2)
match e:
    Add(v): print(v)
    Ask(l, r): print(r)
