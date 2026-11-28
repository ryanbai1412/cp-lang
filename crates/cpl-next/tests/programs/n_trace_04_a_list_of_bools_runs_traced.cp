// expect: ok, output "2 4\n1"
// A traced run gives what a plain run does when a value's type is not `Show`: `List<Bool>` is
// `Show` only where `Bool` is, so the trace shows no value for it, and link instantiates no
// `traced` for it.
fn trues(b: List<Bool>) -> Int = b.count(x => x)
let b = [true, false, true, false]
print(b.count(x => x), b.len)
print(trues([false, true]))
