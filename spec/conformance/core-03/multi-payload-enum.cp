// expect: ok, output "2 -1 12"
// rule: 03.2.5
domain V = 0..3
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
fn f(e: ?Ev) -> Int = match e: None => -1, Some(Add(v)) => Int(v), Some(Ask(l, r)) => 10 + 3 * Int(l) + Int(r)
let xs: List<?Ev> = [Some(Ev.Add(V(2))), None, Some(Ev.Ask(V(0), V(2)))]
print(f(xs[0]), f(xs[1]), f(xs[2]))
