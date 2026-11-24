// expect: ok, output "3 2 YES"
// Lowercase `none` is an ordinary name everywhere a name goes, so a parameter or a field can take it as a `let` can.
struct P:
    none: Int
fn f(none: Int) -> ?Int = if none > 0: Some(none) else: None
let p = P(2)
let q: ?Int = None
print(f(3) ?? 0, p.none, yes(q == None))
