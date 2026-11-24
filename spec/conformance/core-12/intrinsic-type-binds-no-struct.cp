// expect: ok, output "2 1 2"
// rule: 12.14.6
// Loading the module that declares `HashSet<K: Hash> = @intrinsic("hash_set")` leaves `HashSet` the table's type, and a struct's own `Hash` keys it.
use std.derive.{Hash}
struct H(x: Int)
impl Hash for H:
    fn hash(self) -> U64 = U64(0)
let t: HashSet<Int> = HashSet([1, 2, 1])
let u: HashSet<H> = HashSet([H(1), H(1)])
let v: HashSet<H> = HashSet([H(1), H(2)])
print(t.len, u.len, v.len)
