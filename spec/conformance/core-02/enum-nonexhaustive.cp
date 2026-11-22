// expect: E0314 at 4
// rule: 02.7.4
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
fn f(e: Ev) -> Int = match e: Add(v) => v
print(f(Ev.Add(1)))
