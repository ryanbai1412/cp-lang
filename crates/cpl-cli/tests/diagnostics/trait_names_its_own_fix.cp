use std.derive.{Hash, hash_fields}

/// A trait says, in its own words, what a type without an `impl` lacks.
@on_unimplemented("`{T}` can't be shown; write `impl Show2 for {T}`")
trait Show2:
    fn show2(self) -> Str

impl Show2 for Int:
    fn show2(self) -> Str = "int"

fn both<T: Show2>(x: T) -> Str = x.show2()
fn key<T: Hash>(x: T) -> U64 = x.hash()

@allow(unread_field)
struct P(a: Int)

print(both(1), both(true))
print(key(P(1)))
