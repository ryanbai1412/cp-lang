// expect: ok, output "0\nNO"
// rule: R08.1.10
// A lambda calling a method of a type nothing fixes (`K` of `mk()`, whose `by` is `None`) is
// never run, but its call keeps its type: the C++ compiles and every runner agrees.
fn mk<K: Ord>(by: ?(Int -> K) = None) -> ?((Int, Int) -> Bool):
    if let f = by: return Some((a, b) => f(a) < f(b))
    None
if let g = mk(): print(yes(g(1, 2))) else: print(0)
if let g = mk(by: x => -x): print(yes(g(1, 2))) else: print(0)
