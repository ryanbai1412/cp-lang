// expect: ok, output "str val pred pred"
// rule: 08.1.4
// A concrete `self: Str` beats a bare `self: S` whose bounds it meets, and fns whose full signatures no call can take at once overload: `x: T` with `T: Eq` takes no lambda.
fn look(self: Str, pat: Str) -> Str = "str"
fn look<S: Indexed<D, T>, D, T: Eq>(self: S, x: T) -> Str = "val"
fn look<S: Seq<T>, T>(self: S, p: T -> Bool) -> Str = "pred"
print("ab".look("a"), [1, 2].look(1), [1, 2].look(x => x > 1), "ab".look(c => c == 'a'))
