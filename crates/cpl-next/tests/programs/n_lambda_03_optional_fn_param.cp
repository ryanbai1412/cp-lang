// expect: ok, output "4 0 9"
// rule: R02.4.2, R06.1.2
// A lambda passed where `?(A -> R)` is wanted takes its parameter types from the fn inside.
fn ap(f: ?(Int -> Int)) -> Int = if let g = f: g(3) else: 0
fn on(p: ?((Int, Int)) -> Int) -> Int = if let g = p: g((4, 5)) else: 0
print(ap(x => x + 1), ap(None), on((a, b) => a + b))
