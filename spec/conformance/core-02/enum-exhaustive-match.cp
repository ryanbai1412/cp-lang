// expect: ok, output "3"
// rule: 02.7.4
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
fn f(e: Ev) -> Int = match e:
    Add(v) => v
    Ask(l, r) => l + r
print(f(Ev.Ask(1, 2)))
