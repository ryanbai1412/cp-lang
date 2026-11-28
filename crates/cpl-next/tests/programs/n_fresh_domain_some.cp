// expect: ok, output "3 YES"
// rule: R02.9.3
// A body domain returned as the result's `_` is that domain all through the body: the `Some`
// the body builds holds the body's `Box<O>`, and the caller gets a domain of its own.
struct Box<P>:
    at: [P]Int
fn maybe(k: Int) -> ?Box<_>:
    if k == 0: return None
    domain O = 0..k
    Some(Box<O>(at: [O] _ => 1))
let b = maybe(3)!
print(b.at.len, yes(maybe(0) == None))
