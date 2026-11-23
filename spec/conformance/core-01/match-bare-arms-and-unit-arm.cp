// expect: ok, output "3"
// rule: 01.7.3
// Arms name variants bare, and `()` is the arm that does nothing (rebuild round 19 H08).
enum Ev = Enter(v: Int) | Leave(v: Int)
var s = 0
for e in [Ev.Enter(3), Ev.Leave(4)]:
    match e:
        Enter(v) => s += v
        Leave(_) => ()
print(s)
