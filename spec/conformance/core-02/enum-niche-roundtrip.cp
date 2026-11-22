// expect: ok, output "3 13"
// rule: 02.7.5
domain V = 0..4
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
fn f(e: Ev) -> Int = match e: Add(v) => Int(v), Ask(l, r) => 10 * Int(l) + Int(r)
print(f(Ev.Add(V(3))), f(Ev.Ask(V(1), V(3))))
