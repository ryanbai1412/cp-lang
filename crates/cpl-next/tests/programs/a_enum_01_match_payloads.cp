// expect: ok, output "5\n34\n-1"
// rule: R02.7.1, R02.7.2
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
    Stop
fn f(e: Ev) -> Int = match e:
    Add(v) => v
    Ask(l, r) => l * 10 + r
    Stop => -1
print(f(Ev.Add(5)))
print(f(Ev.Ask(3, 4)))
print(f(Ev.Stop))
