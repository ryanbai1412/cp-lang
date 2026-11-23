// A `.rev` given to a generic fn keeps walking backwards in the callee:
// its keys come in reversed order, also through a nested generic call, a
// `let` bound to the `.rev`, and `self`; the same fn given the list itself
// is its own instance and walks forwards (R07.4.7).
fn kk<S: Sized>(s: S) -> List<Int> = List(s.keys)
fn outer<S: Sized>(s: S) -> List<Int> = kk(s)
fn last_key<S: Sized>(self: S) -> Int = self.keys.nth(self.len - 1)
let a = [10, 20, 30]
print(kk(a.rev))
print(kk(a))
print(outer(a.rev))
let r = a.rev
print(kk(r), outer(a))
print(a.rev.last_key(), a.last_key())
