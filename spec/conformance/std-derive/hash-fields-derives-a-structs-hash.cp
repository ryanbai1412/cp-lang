// expect: ok, output "1 0 1 1\n4"
// rule: std.derive.hash_fields
// `Hash` is a std trait (decision 256): `hash_fields` is a struct's whole
// derive, built-in types have `hash()`, and `T: Hash` bounds a generic.
use std.derive.{hash_fields}
struct P(a: Int, b: Str)
impl Hash for P: fn hash(self) -> U64 = hash_fields(self)
fn same<T: Hash>(a: T, b: T) -> Bool = a.hash() == b.hash()
print(Int(same(P(1, "x"), P(1, "x"))), Int(same(P(1, "x"), P(2, "x"))), Int("ab".hash() == "ab".hash()), Int(same(3, 3)))
var m: HashMap<P, Int> = HashMap()
m[P(1, "x")] = 4
print(m[P(1, "x")])
