// Inside `if T: Add:` every value of type `T` is an `Add`: an `if let`
// binding, an operator and a generic call that requires the bound.
fn plus<T: Add>(a: T, b: T) -> T = a + b
fn bump<T>(o: ?T, d: T) -> ?T:
    if T: Add:
        if let x = o: return Some(plus(x, d) + d)
    o
print(bump(Some(2), 3)!, bump(Some("a"), "b")!, yes(bump(Some(true), false)!))
